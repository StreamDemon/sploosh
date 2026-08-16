//! Recursive-descent parser for the compiler bootstrap.

use sploosh_ast::*;
use sploosh_lexer::{Keyword, LexError, Token, TokenKind, lex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub fn parse_program(source: &str) -> Result<Program, Vec<ParseError>> {
    let tokens = lex(source).map_err(lex_errors)?;
    Parser::new(tokens, source).parse_program()
}

fn lex_errors(errors: Vec<LexError>) -> Vec<ParseError> {
    errors
        .into_iter()
        .map(|err| ParseError {
            message: err.message,
            span: err.span,
        })
        .collect()
}

/// Parser-internal classification of infix operators: `=` builds an
/// `ExprKind::Assign` node, everything else an `ExprKind::Binary`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Infix {
    Assign,
    Op(BinaryOp),
}

struct Parser<'src> {
    tokens: Vec<Token>,
    source: &'src str,
    pos: usize,
    errors: Vec<ParseError>,
    /// When set, a `struct_literal` may not be the outermost expression — the
    /// block-head restriction (§5.1, §5.2; §16 `struct_literal` side condition).
    no_struct_literal: bool,
}

impl<'src> Parser<'src> {
    fn new(tokens: Vec<Token>, source: &'src str) -> Self {
        Self {
            tokens,
            source,
            pos: 0,
            errors: Vec::new(),
            no_struct_literal: false,
        }
    }

    /// Source text for a token; tokens carry only spans (see `Token::text`).
    fn text(&self, token: &Token) -> &'src str {
        token.text(self.source)
    }

    fn parse_program(mut self) -> Result<Program, Vec<ParseError>> {
        let mut items = Vec::new();
        while !self.eof() {
            self.skip_doc_comments();
            match self.item() {
                Some(item) => items.push(item),
                None => self.recover_item(),
            }
        }
        if self.errors.is_empty() {
            Ok(Program { items })
        } else {
            Err(self.errors)
        }
    }

    fn item(&mut self) -> Option<Item> {
        self.skip_doc_comments();
        let start = self.peek()?.span.start;
        let attrs = self.attrs();
        let pub_token = self.eat_keyword(Keyword::Pub);
        let offchain_token = self.eat_keyword(Keyword::Offchain);
        let async_token = self.eat_keyword(Keyword::Async);
        let visibility = if pub_token.is_some() {
            Visibility::Public
        } else {
            Visibility::Private
        };
        let is_offchain = offchain_token.is_some();
        let is_async = async_token.is_some();
        // §16: `offchain`/`async` prefix only `fn_def`; `pub` prefixes every
        // item form except `impl_block`, `actor_def`, `onchain_mod`/`event_def`,
        // and `extern_block`.
        let next = self.peek_kind()?;
        if !matches!(next, TokenKind::Keyword(Keyword::Fn)) {
            if let Some(token) = &offchain_token {
                self.error_at(token.span, "`offchain` applies only to `fn` items");
            }
            if let Some(token) = &async_token {
                self.error_at(token.span, "`async` applies only to `fn` items");
            }
        }
        if let Some(token) = &pub_token
            && matches!(
                next,
                TokenKind::Keyword(
                    Keyword::Impl | Keyword::Actor | Keyword::Onchain | Keyword::Extern
                )
            )
        {
            self.error_at(
                token.span,
                "`pub` is not allowed on `impl`, `actor`, `onchain`, or `extern` items",
            );
        }
        let kind = match next {
            TokenKind::Keyword(Keyword::Fn) => ItemKind::Function(self.function_after_mods(
                visibility,
                is_async,
                is_offchain,
                true,
            )?),
            TokenKind::Keyword(Keyword::Struct) => ItemKind::Struct(self.struct_def()?),
            TokenKind::Keyword(Keyword::Enum) => ItemKind::Enum(self.enum_def(false)?),
            TokenKind::Keyword(Keyword::Actor) => ItemKind::Actor(self.actor_def()?),
            TokenKind::Keyword(Keyword::Mod) => ItemKind::Module(self.mod_def()?),
            TokenKind::Keyword(Keyword::Use) => ItemKind::Use(self.use_stmt()?),
            TokenKind::Keyword(Keyword::Const) => ItemKind::Const(self.const_def()?),
            TokenKind::Keyword(Keyword::Type) => ItemKind::TypeAlias(self.type_alias()?),
            TokenKind::Keyword(Keyword::Trait) => ItemKind::Trait(self.trait_def()?),
            TokenKind::Keyword(Keyword::Impl) => ItemKind::Impl(self.impl_block()?),
            TokenKind::Keyword(Keyword::Onchain) => self.onchain_item()?,
            TokenKind::Keyword(Keyword::Extern) => ItemKind::ExternBlock(self.extern_block()?),
            _ => {
                self.error_here("expected item");
                return None;
            }
        };
        let end = self.prev_span().end;
        Some(Item {
            attrs,
            visibility,
            kind,
            span: Span::new(start, end),
        })
    }

    fn attrs(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while let Some(at) = self.eat(TokenKind::At) {
            if let Some(name) = self.ident() {
                let mut args = Vec::new();
                let mut end = name.span.end;
                if self.eat(TokenKind::LParen).is_some() {
                    args = self.attr_args();
                    end = match self.expect(TokenKind::RParen) {
                        Some(close) => close.span.end,
                        None => self.prev_span().end,
                    };
                }
                attrs.push(Attribute {
                    name,
                    args,
                    span: Span::new(at.span.start, end),
                });
            }
        }
        attrs
    }

    /// `attr_args = attr_arg { "," attr_arg }` (§16).
    fn attr_args(&mut self) -> Vec<AttrArg> {
        let mut args = Vec::new();
        while !self.at(TokenKind::RParen) && !self.eof() {
            match self.attr_arg() {
                Some(arg) => args.push(arg),
                None => self.recover_until(&[TokenKind::Comma, TokenKind::RParen]),
            }
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        args
    }

    /// `attr_arg = IDENT [ ":" expr | "=" expr | "(" expr ")" ] | expr` (§16).
    /// Only `IDENT ":"` needs lookahead — `:` cannot continue an expression.
    /// The `=` and `(...)` alternatives are canonicalized out of the parsed
    /// expression, since both are valid expression shapes themselves.
    fn attr_arg(&mut self) -> Option<AttrArg> {
        if self.at(TokenKind::Ident) && self.peek_kind_at(1) == Some(TokenKind::Colon) {
            let name = self.ident()?;
            self.bump();
            let value = self.delimited_expr()?;
            return Some(AttrArg::Named { name, value });
        }
        Some(classify_attr_expr(self.delimited_expr()?))
    }

    fn function_after_mods(
        &mut self,
        visibility: Visibility,
        is_async: bool,
        is_offchain: bool,
        body: bool,
    ) -> Option<Function> {
        self.expect_keyword(Keyword::Fn)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        self.expect(TokenKind::LParen)?;
        let params = self.params();
        self.expect(TokenKind::RParen)?;
        let return_type = if self.eat(TokenKind::Arrow).is_some() {
            Some(self.ty()?)
        } else {
            None
        };
        self.maybe_where_clause();
        let body = if body {
            Some(self.block()?)
        } else {
            self.expect(TokenKind::Semi)?;
            None
        };
        Some(Function {
            name,
            visibility,
            is_async,
            is_offchain,
            params,
            return_type,
            body,
        })
    }

    fn params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.eof() {
            if let Some(param) = self.param() {
                params.push(param);
            }
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        params
    }

    fn param(&mut self) -> Option<Param> {
        let start = self.peek()?.span.start;
        let by_ref = self.eat(TokenKind::Amp).is_some();
        let mutable = by_ref && self.eat_ident_text("mut").is_some();
        if self.eat_keyword(Keyword::SelfValue).is_some() {
            let end = self.prev_span().end;
            return Some(Param::Receiver {
                mutable,
                by_ref,
                span: Span::new(start, end),
            });
        }
        if by_ref {
            self.error_here("receiver reference must be followed by `self`");
            return None;
        }
        let name = self.ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.ty()?;
        Some(Param::Named { name, ty })
    }

    fn struct_def(&mut self) -> Option<Struct> {
        self.expect_keyword(Keyword::Struct)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        self.maybe_where_clause();
        let fields = self.field_block()?;
        Some(Struct { name, fields })
    }

    fn field_block(&mut self) -> Option<Vec<Field>> {
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            self.skip_doc_comments();
            let visibility = if self.eat_keyword(Keyword::Pub).is_some() {
                Visibility::Public
            } else {
                Visibility::Private
            };
            let Some(name) = self.ident() else {
                self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                let _ = self.eat(TokenKind::Comma);
                continue;
            };
            self.expect(TokenKind::Colon)?;
            let ty = self.ty()?;
            fields.push(Field {
                name,
                ty,
                visibility,
            });
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Some(fields)
    }

    fn enum_def(&mut self, onchain: bool) -> Option<Enum> {
        self.expect_keyword(Keyword::Enum)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        self.maybe_where_clause();
        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            if let Some(name) = self.ident() {
                let kind = if self.eat(TokenKind::LParen).is_some() {
                    let mut types = Vec::new();
                    while !self.at(TokenKind::RParen) && !self.eof() {
                        types.push(self.ty()?);
                        if self.eat(TokenKind::Comma).is_none() {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    VariantKind::Tuple(types)
                } else if self.at(TokenKind::LBrace) {
                    VariantKind::Struct(self.field_block()?)
                } else {
                    VariantKind::Unit
                };
                variants.push(Variant { name, kind });
            }
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Some(Enum {
            name,
            variants,
            onchain,
        })
    }

    fn actor_def(&mut self) -> Option<Actor> {
        self.expect_keyword(Keyword::Actor)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        self.maybe_where_clause();
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut handlers = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            self.skip_doc_comments();
            // A handler is a `fn_def`, so its attrs (`@mailbox(...)`, ...) are
            // preserved. Fields take no attrs in §16; anything parsed before a
            // field is currently discarded, matching the item-position
            // tolerance for attrs on kinds the grammar leaves bare.
            let attrs = self.attrs();
            let visibility = if self.eat_keyword(Keyword::Pub).is_some() {
                Visibility::Public
            } else {
                Visibility::Private
            };
            let is_async = self.eat_keyword(Keyword::Async).is_some();
            if self.at_keyword(Keyword::Fn) {
                let function = self.function_after_mods(visibility, is_async, false, true)?;
                handlers.push(Handler { attrs, function });
            } else {
                let name = self.ident()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.ty()?;
                fields.push(Field {
                    name,
                    ty,
                    visibility,
                });
                let _ = self.eat(TokenKind::Comma);
            }
        }
        self.expect(TokenKind::RBrace)?;
        Some(Actor {
            name,
            fields,
            handlers,
        })
    }

    fn mod_def(&mut self) -> Option<Module> {
        self.expect_keyword(Keyword::Mod)?;
        let name = self.ident()?;
        if self.eat(TokenKind::Semi).is_some() {
            return Some(Module {
                name,
                items: Vec::new(),
                inline: false,
            });
        }
        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            self.skip_doc_comments();
            if let Some(item) = self.item() {
                items.push(item);
            } else {
                self.recover_item();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Some(Module {
            name,
            items,
            inline: true,
        })
    }

    fn use_stmt(&mut self) -> Option<Use> {
        self.expect_keyword(Keyword::Use)?;
        let path = self.path()?;
        if self.at(TokenKind::ColonColon) {
            self.bump();
            if self.at(TokenKind::LBrace) {
                self.skip_balanced(TokenKind::LBrace, TokenKind::RBrace);
            }
        }
        self.expect(TokenKind::Semi)?;
        Some(Use { path })
    }

    fn const_def(&mut self) -> Option<Const> {
        self.expect_keyword(Keyword::Const)?;
        let name = self.ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.ty()?;
        self.expect(TokenKind::Eq)?;
        let value = self.expr(0)?;
        self.expect(TokenKind::Semi)?;
        Some(Const { name, ty, value })
    }

    fn type_alias(&mut self) -> Option<TypeAlias> {
        self.expect_keyword(Keyword::Type)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        self.expect(TokenKind::Eq)?;
        let ty = self.ty()?;
        self.expect(TokenKind::Semi)?;
        Some(TypeAlias { name, ty })
    }

    fn trait_def(&mut self) -> Option<Trait> {
        self.expect_keyword(Keyword::Trait)?;
        let name = self.ident()?;
        self.maybe_generic_params();
        if self.eat(TokenKind::Colon).is_some() {
            self.bounds();
        }
        self.maybe_where_clause();
        self.skip_item_body();
        Some(Trait { name })
    }

    /// `bounds = bound { "+" bound }` with `bound = trait_ref | lifetime` (§16).
    /// Parsed for validation and diagnostics; not stored during bootstrap.
    fn bounds(&mut self) {
        loop {
            if self.eat(TokenKind::Lifetime).is_none() && self.trait_ref().is_none() {
                self.recover_until(&[TokenKind::LBrace, TokenKind::Semi]);
                return;
            }
            if self.eat(TokenKind::Plus).is_none() {
                break;
            }
        }
    }

    /// `trait_ref = type_path [ type_args ]` (§16).
    fn trait_ref(&mut self) -> Option<TraitRef> {
        let path = self.path()?;
        let args = self.type_args();
        Some(TraitRef { path, args })
    }

    fn impl_block(&mut self) -> Option<ImplBlock> {
        self.expect_keyword(Keyword::Impl)?;
        self.maybe_generic_params();
        let first = self.ty()?;
        let (trait_ref, target) = if self.eat_keyword(Keyword::For).is_some() {
            let Type::Path { path, args } = first else {
                self.error_here("expected trait path before `for`");
                return None;
            };
            (Some(TraitRef { path, args }), self.ty()?)
        } else {
            (None, first)
        };
        self.maybe_where_clause();
        self.skip_item_body();
        Some(ImplBlock { trait_ref, target })
    }

    fn onchain_item(&mut self) -> Option<ItemKind> {
        self.expect_keyword(Keyword::Onchain)?;
        if self.eat_keyword(Keyword::Mod).is_some() {
            let name = self.ident()?;
            self.expect(TokenKind::LBrace)?;
            let mut items = Vec::new();
            while !self.at(TokenKind::RBrace) && !self.eof() {
                self.skip_doc_comments();
                if self.eat_ident_text("storage").is_some() {
                    self.skip_balanced(TokenKind::LBrace, TokenKind::RBrace);
                    continue;
                }
                if let Some(item) = self.item() {
                    items.push(item);
                } else {
                    self.recover_item();
                }
            }
            self.expect(TokenKind::RBrace)?;
            Some(ItemKind::OnchainModule(OnchainModule { name, items }))
        } else if self.at_keyword(Keyword::Enum) {
            Some(ItemKind::Enum(self.enum_def(true)?))
        } else {
            self.error_here("expected `mod` or `enum` after `onchain`");
            None
        }
    }

    fn extern_block(&mut self) -> Option<ExternBlock> {
        self.expect_keyword(Keyword::Extern)?;
        let target = if self.at(TokenKind::StringLit) {
            let token = self.bump();
            self.text(&token).to_string()
        } else {
            self.expect_keyword(Keyword::Onchain)?;
            self.expect_keyword(Keyword::Mod)?;
            self.ident()?.name
        };
        let _async = self.eat_keyword(Keyword::Async);
        self.expect(TokenKind::LBrace)?;
        let mut functions = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            self.skip_doc_comments();
            let visibility = if self.eat_keyword(Keyword::Pub).is_some() {
                Visibility::Public
            } else {
                Visibility::Private
            };
            functions.push(self.function_after_mods(visibility, false, false, false)?);
        }
        self.expect(TokenKind::RBrace)?;
        Some(ExternBlock { target, functions })
    }

    fn ty(&mut self) -> Option<Type> {
        if self.eat(TokenKind::Amp).is_some() {
            let _lifetime = self.eat(TokenKind::Lifetime);
            let mutable = self.eat_ident_text("mut").is_some();
            let inner = self.ty()?;
            return Some(Type::Reference {
                mutable,
                inner: Box::new(inner),
            });
        }
        if self.eat(TokenKind::LBracket).is_some() {
            let inner = self.ty()?;
            if self.eat(TokenKind::Semi).is_some() {
                let len = self.expr(0)?;
                self.expect(TokenKind::RBracket)?;
                return Some(Type::Array {
                    inner: Box::new(inner),
                    len: Box::new(len),
                });
            }
            self.expect(TokenKind::RBracket)?;
            return Some(Type::Slice(Box::new(inner)));
        }
        if self.eat(TokenKind::LParen).is_some() {
            let mut types = Vec::new();
            while !self.at(TokenKind::RParen) && !self.eof() {
                types.push(self.ty()?);
                if self.eat(TokenKind::Comma).is_none() {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            return Some(Type::Tuple(types));
        }
        if self.eat_keyword(Keyword::Fn).is_some() {
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            while !self.at(TokenKind::RParen) && !self.eof() {
                params.push(self.ty()?);
                if self.eat(TokenKind::Comma).is_none() {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            self.expect(TokenKind::Arrow)?;
            let ret = self.ty()?;
            return Some(Type::Function {
                params,
                ret: Box::new(ret),
            });
        }
        if self.eat_ident_text("dyn").is_some() {
            return Some(Type::Dyn(self.trait_ref()?));
        }
        let path = self.path()?;
        let args = self.type_args();
        if path.segments.len() == 1 && args.is_empty() && is_primitive_type(&path.segments[0]) {
            Some(Type::Primitive(path.segments[0].clone()))
        } else {
            Some(Type::Path { path, args })
        }
    }

    fn block(&mut self) -> Option<Block> {
        // A block is always a fresh scope: the struct-literal block-head
        // restriction (§5.1) must not leak into its body, even when the block
        // belongs to an `if`/`while` nested inside a restricted condition.
        let prev = self.no_struct_literal;
        self.no_struct_literal = false;
        let result = self.block_inner();
        self.no_struct_literal = prev;
        result
    }

    fn block_inner(&mut self) -> Option<Block> {
        let start = self.expect(TokenKind::LBrace)?.span.start;
        let mut statements = Vec::new();
        let mut tail = None;
        while !self.at(TokenKind::RBrace) && !self.eof() {
            self.skip_doc_comments();
            if self.eat_keyword(Keyword::Let).is_some() {
                let _mutable = self.eat_ident_text("mut");
                let name = self.ident()?.name;
                let ty = if self.eat(TokenKind::Colon).is_some() {
                    Some(self.ty()?)
                } else {
                    None
                };
                self.expect(TokenKind::Eq)?;
                let value = self.expr(0)?;
                self.expect(TokenKind::Semi)?;
                statements.push(Stmt::Let { name, ty, value });
            } else if self.eat_keyword(Keyword::Return).is_some() {
                let value = if self.at(TokenKind::Semi) {
                    None
                } else {
                    Some(self.expr(0)?)
                };
                self.expect(TokenKind::Semi)?;
                statements.push(Stmt::Return(value));
            } else if self.eat_keyword(Keyword::Break).is_some() {
                self.expect(TokenKind::Semi)?;
                statements.push(Stmt::Break);
            } else if self.eat_keyword(Keyword::Continue).is_some() {
                self.expect(TokenKind::Semi)?;
                statements.push(Stmt::Continue);
            } else if self.at_ident_text("send") && self.peek_kind_at(1).is_some_and(can_begin_expr)
            {
                // §2.7: `send` at statement head followed by any token that can
                // begin an expression always opens a send-statement; the operand
                // must be a method call (`handle.method(args)`, §8.2). Before any
                // other token, `send` stays an ordinary identifier.
                self.bump();
                let expr = self.expr(0)?;
                if !is_method_call(&expr) {
                    self.error_at(
                        expr.span,
                        "`send` operand must be a method call on a handle",
                    );
                }
                self.expect(TokenKind::Semi)?;
                statements.push(Stmt::Expr(expr));
            } else {
                let expr = self.expr(0)?;
                if self.eat(TokenKind::Semi).is_some() {
                    statements.push(Stmt::Expr(expr));
                } else {
                    tail = Some(Box::new(expr));
                    break;
                }
            }
        }
        let end = self.expect(TokenKind::RBrace)?.span.end;
        Some(Block {
            statements,
            tail,
            span: Span::new(start, end),
        })
    }

    fn expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.prefix()?;
        loop {
            if self.eat(TokenKind::Question).is_some() {
                let span = lhs.span.join(self.prev_span());
                lhs = Expr {
                    kind: ExprKind::ErrorProp(Box::new(lhs)),
                    span,
                };
                continue;
            }
            if self.eat(TokenKind::Dot).is_some() {
                if self.eat_keyword(Keyword::Await).is_some() {
                    let span = lhs.span.join(self.prev_span());
                    lhs = Expr {
                        kind: ExprKind::Await(Box::new(lhs)),
                        span,
                    };
                } else {
                    let name = self.ident()?.name;
                    let span = lhs.span.join(self.prev_span());
                    lhs = Expr {
                        kind: ExprKind::Field {
                            base: Box::new(lhs),
                            name,
                        },
                        span,
                    };
                }
                continue;
            }
            if self.eat(TokenKind::LParen).is_some() {
                let type_args = Vec::new();
                let args = self.args(TokenKind::RParen)?;
                let end = self.expect(TokenKind::RParen)?.span.end;
                let span = Span::new(lhs.span.start, end);
                lhs = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(lhs),
                        type_args,
                        args,
                    },
                    span,
                };
                continue;
            }
            if let Some(type_args) = self.turbofish() {
                self.expect(TokenKind::LParen)?;
                let args = self.args(TokenKind::RParen)?;
                let end = self.expect(TokenKind::RParen)?.span.end;
                let span = Span::new(lhs.span.start, end);
                lhs = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(lhs),
                        type_args,
                        args,
                    },
                    span,
                };
                continue;
            }
            if self.eat(TokenKind::LBracket).is_some() {
                let index = self.delimited_expr()?;
                let end = self.expect(TokenKind::RBracket)?.span.end;
                let span = Span::new(lhs.span.start, end);
                lhs = Expr {
                    kind: ExprKind::Index {
                        base: Box::new(lhs),
                        index: Box::new(index),
                    },
                    span,
                };
                continue;
            }
            if self.eat_keyword(Keyword::As).is_some() {
                let ty = self.ty()?;
                let span = lhs.span.join(self.prev_span());
                lhs = Expr {
                    kind: ExprKind::Cast {
                        expr: Box::new(lhs),
                        ty,
                    },
                    span,
                };
                continue;
            }
            let Some((infix, left_bp, right_bp)) = self.infix_binding_power() else {
                break;
            };
            if left_bp < min_bp {
                break;
            }
            self.bump();
            if infix == Infix::Op(BinaryOp::Pipe) {
                // §16: the RHS of `|>` is a `pipe_stage`, not a precedence-climbed
                // expression.
                let stage = self.pipe_stage()?;
                let span = lhs.span.join(stage.span);
                lhs = Expr {
                    kind: ExprKind::Binary {
                        op: BinaryOp::Pipe,
                        left: Box::new(lhs),
                        right: Box::new(stage),
                    },
                    span,
                };
                // §5.7: a stage's trailing `?` applies to the accumulated pipe
                // application result — `x |> f?` is `(x |> f)?`.
                if self.eat(TokenKind::Question).is_some() {
                    let span = lhs.span.join(self.prev_span());
                    lhs = Expr {
                        kind: ExprKind::ErrorProp(Box::new(lhs)),
                        span,
                    };
                }
                continue;
            }
            let rhs = self.expr(right_bp)?;
            let span = lhs.span.join(rhs.span);
            lhs = match infix {
                Infix::Assign => {
                    // §16: only an `assign_target` may appear on the left side.
                    if !is_assign_target(&lhs) {
                        self.error_at(lhs.span, "invalid assignment target");
                    }
                    Expr {
                        kind: ExprKind::Assign {
                            target: Box::new(lhs),
                            value: Box::new(rhs),
                        },
                        span,
                    }
                }
                Infix::Op(op) => Expr {
                    kind: ExprKind::Binary {
                        op,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                },
            };
            // The precedence table marks `..`/`..=` non-associative: a range
            // operand may not itself be an unparenthesized range.
            if matches!(infix, Infix::Op(BinaryOp::Range | BinaryOp::RangeInclusive))
                && matches!(
                    self.peek_kind(),
                    Some(TokenKind::DotDot | TokenKind::DotDotEq)
                )
            {
                self.error_here("range operators cannot be chained; parenthesize one side");
            }
        }
        Some(lhs)
    }

    /// Condition / scrutinee expression with the struct-literal block-head
    /// restriction active: a `struct_literal` may not be the outermost
    /// expression of an `if`/`while` condition, `match` scrutinee, or `for`
    /// iterable (LANGUAGE_SPEC §5.1, §5.2; §16 `struct_literal` side condition).
    /// Parenthesize to use one there.
    fn cond_expr(&mut self) -> Option<Expr> {
        let prev = self.no_struct_literal;
        self.no_struct_literal = true;
        let result = self.expr(0);
        self.no_struct_literal = prev;
        result
    }

    /// Expression inside a delimited group (parens, call args, index, `vec!`),
    /// where struct literals are allowed again — the restriction only binds the
    /// outermost expression, not anything nested inside brackets.
    fn delimited_expr(&mut self) -> Option<Expr> {
        let prev = self.no_struct_literal;
        self.no_struct_literal = false;
        let result = self.expr(0);
        self.no_struct_literal = prev;
        result
    }

    fn prefix(&mut self) -> Option<Expr> {
        let token = *self.peek()?;
        match token.kind {
            TokenKind::IntLit
            | TokenKind::FloatLit
            | TokenKind::StringLit
            | TokenKind::CharLit
            | TokenKind::Keyword(Keyword::True | Keyword::False) => {
                self.bump();
                Some(Expr {
                    kind: ExprKind::Literal(self.literal_from_token(&token)),
                    span: token.span,
                })
            }
            TokenKind::Ident if self.text(&token) == "vec" => {
                self.bump();
                if self.eat(TokenKind::Bang).is_some() {
                    // §16 `vec_literal`: `vec` "!" only ever binds to square
                    // brackets — `vec!(...)` / `vec!{...}` are parse errors, not
                    // a silent fallback to a plain `vec` path.
                    if self.eat(TokenKind::LBracket).is_none() {
                        self.error_here("expected `[` after `vec!`");
                        return None;
                    }
                    if self.eat(TokenKind::RBracket).is_some() {
                        return Some(Expr {
                            kind: ExprKind::VecLiteral(Vec::new()),
                            span: Span::new(token.span.start, self.prev_span().end),
                        });
                    }
                    let first = self.delimited_expr()?;
                    if self.eat(TokenKind::Semi).is_some() {
                        let count = self.delimited_expr()?;
                        let end = self.expect(TokenKind::RBracket)?.span.end;
                        return Some(Expr {
                            kind: ExprKind::VecRepeat {
                                value: Box::new(first),
                                count: Box::new(count),
                            },
                            span: Span::new(token.span.start, end),
                        });
                    }
                    let mut items = vec![first];
                    while self.eat(TokenKind::Comma).is_some() && !self.at(TokenKind::RBracket) {
                        items.push(self.delimited_expr()?);
                    }
                    let end = self.expect(TokenKind::RBracket)?.span.end;
                    return Some(Expr {
                        kind: ExprKind::VecLiteral(items),
                        span: Span::new(token.span.start, end),
                    });
                }
                Some(Expr {
                    kind: ExprKind::Path(Path {
                        segments: vec![self.text(&token).to_string()],
                        span: token.span,
                    }),
                    span: token.span,
                })
            }
            TokenKind::Ident | TokenKind::Keyword(Keyword::SelfValue | Keyword::SelfType) => {
                let path = self.path()?;
                if self.at(TokenKind::LBrace) && !self.no_struct_literal {
                    self.bump();
                    let fields = self.field_inits()?;
                    let end = self.expect(TokenKind::RBrace)?.span.end;
                    return Some(Expr {
                        kind: ExprKind::StructLiteral { path, fields },
                        span: Span::new(token.span.start, end),
                    });
                }
                Some(Expr {
                    span: path.span,
                    kind: ExprKind::Path(path),
                })
            }
            TokenKind::Bang | TokenKind::Minus | TokenKind::Star | TokenKind::Amp => {
                let op = match token.kind {
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Star => UnaryOp::Deref,
                    TokenKind::Amp => UnaryOp::Ref,
                    _ => unreachable!(),
                };
                self.bump();
                if op == UnaryOp::Ref {
                    let _mutable = self.eat_ident_text("mut");
                }
                let expr = self.expr(11)?;
                let span = token.span.join(expr.span);
                Some(Expr {
                    kind: ExprKind::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                    span,
                })
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.delimited_expr()?;
                let end = self.expect(TokenKind::RParen)?.span.end;
                Some(Expr {
                    span: Span::new(token.span.start, end),
                    ..expr
                })
            }
            TokenKind::LBrace => self.block().map(|block| Expr {
                span: block.span,
                kind: ExprKind::Block(block),
            }),
            TokenKind::Keyword(Keyword::If) => {
                if self.peek_kind_at(1) == Some(TokenKind::Keyword(Keyword::Let)) {
                    self.if_let_expr()
                } else {
                    self.if_expr()
                }
            }
            TokenKind::Keyword(Keyword::Match) => self.match_expr(),
            TokenKind::Keyword(Keyword::While) => self.while_expr(),
            TokenKind::Keyword(Keyword::For) => self.for_expr(),
            TokenKind::Keyword(Keyword::Loop) => self.loop_expr(),
            _ => {
                self.error_here("expected expression");
                None
            }
        }
    }

    /// Canonicalizes a literal token (`IntLit`/`FloatLit`/`StringLit`/
    /// `CharLit`, `true`/`false`) to its `Literal` node (§16 `literal`) —
    /// shared by the expression and pattern grammars so both stay aligned.
    fn literal_from_token(&self, token: &Token) -> Literal {
        match token.kind {
            TokenKind::IntLit => Literal::Int(self.text(token).to_string()),
            TokenKind::FloatLit => Literal::Float(self.text(token).to_string()),
            TokenKind::StringLit => Literal::String(self.text(token).to_string()),
            TokenKind::CharLit => Literal::Char(self.text(token).to_string()),
            TokenKind::Keyword(Keyword::True) => Literal::Bool(true),
            TokenKind::Keyword(Keyword::False) => Literal::Bool(false),
            _ => unreachable!("literal_from_token on a non-literal token"),
        }
    }

    /// `pipe_stage = stage_callee [ "(" args ")" ]` with
    /// `stage_callee = path_expr [ turbofish ] { "." IDENT [ turbofish ] }` (§16).
    /// The stage-trailing `?` is consumed by the caller so it can wrap the
    /// accumulated pipe application (§5.7). The `"(" closure ")"` stage form is
    /// not yet implemented — closures have no parse production.
    fn pipe_stage(&mut self) -> Option<Expr> {
        if self.at(TokenKind::LParen) {
            self.error_here("closure pipe stages are not yet implemented");
            return None;
        }
        if !matches!(
            self.peek_kind(),
            Some(TokenKind::Ident | TokenKind::Keyword(Keyword::SelfValue | Keyword::SelfType))
        ) {
            self.error_here("expected pipe stage: a function path or method chain");
            return None;
        }
        let path = self.path()?;
        let mut callee = Expr {
            span: path.span,
            kind: ExprKind::Path(path),
        };
        let mut type_args = self.turbofish();
        while type_args.is_none() && self.eat(TokenKind::Dot).is_some() {
            let name = self.ident()?.name;
            let span = callee.span.join(self.prev_span());
            callee = Expr {
                kind: ExprKind::Field {
                    base: Box::new(callee),
                    name,
                },
                span,
            };
            type_args = self.turbofish();
        }
        if type_args.is_some() && self.at(TokenKind::Dot) {
            self.error_here("turbofish on a non-final pipe-stage segment is not yet implemented");
            return None;
        }
        if self.eat(TokenKind::LParen).is_some() {
            let args = self.args(TokenKind::RParen)?;
            let end = self.expect(TokenKind::RParen)?.span.end;
            let span = Span::new(callee.span.start, end);
            callee = Expr {
                kind: ExprKind::Call {
                    callee: Box::new(callee),
                    type_args: type_args.unwrap_or_default(),
                    args,
                },
                span,
            };
        } else if let Some(type_args) = type_args {
            // `x |> parse::<i64>` has no parens; keep the type args on a zero-arg
            // call — §5.6 desugaring makes this identical to `x |> parse::<i64>()`.
            let span = callee.span.join(self.prev_span());
            callee = Expr {
                kind: ExprKind::Call {
                    callee: Box::new(callee),
                    type_args,
                    args: Vec::new(),
                },
                span,
            };
        }
        Some(callee)
    }

    fn if_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::If)?.span.start;
        let condition = self.cond_expr()?;
        let then_block = self.block()?;
        let else_branch = if self.eat_keyword(Keyword::Else).is_some() {
            Some(Box::new(if self.at_keyword(Keyword::If) {
                // §16: the else of an `if_expr` may chain into another
                // `if_expr` or an `if_let_expr`.
                if self.peek_kind_at(1) == Some(TokenKind::Keyword(Keyword::Let)) {
                    self.if_let_expr()?
                } else {
                    self.if_expr()?
                }
            } else {
                let block = self.block()?;
                Expr {
                    span: block.span,
                    kind: ExprKind::Block(block),
                }
            }))
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map_or(then_block.span.end, |expr| expr.span.end);
        Some(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_block,
                else_branch,
            },
            span: Span::new(start, end),
        })
    }

    /// `if_let_expr = "if" "let" pattern "=" expr block [ "else" block ]`
    /// (§16). The else is a plain block only — no `else if` chains. Like the
    /// other condition-like positions, the scrutinee parses under the
    /// block-head struct-literal restriction (see #89, item 3).
    fn if_let_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::If)?.span.start;
        self.expect_keyword(Keyword::Let)?;
        let pattern = self.pattern();
        self.expect(TokenKind::Eq)?;
        let scrutinee = self.cond_expr()?;
        let then_block = self.block()?;
        let else_block = if self.eat_keyword(Keyword::Else).is_some() {
            Some(self.block()?)
        } else {
            None
        };
        let end = else_block
            .as_ref()
            .map_or(then_block.span.end, |b| b.span.end);
        Some(Expr {
            kind: ExprKind::IfLet {
                pattern: pattern?,
                scrutinee: Box::new(scrutinee),
                then_block,
                else_block,
            },
            span: Span::new(start, end),
        })
    }

    /// `"while" expr block | while_let_expr = "while" "let" pattern "=" expr
    /// block` (§16). The condition and the while-let scrutinee both parse
    /// under the block-head struct-literal restriction (§5.1; #89 item 3).
    fn while_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::While)?.span.start;
        if self.eat_keyword(Keyword::Let).is_some() {
            let pattern = self.pattern();
            self.expect(TokenKind::Eq)?;
            let scrutinee = self.cond_expr()?;
            let body = self.block()?;
            let end = body.span.end;
            return Some(Expr {
                kind: ExprKind::WhileLet {
                    pattern: pattern?,
                    scrutinee: Box::new(scrutinee),
                    body,
                },
                span: Span::new(start, end),
            });
        }
        let condition = self.cond_expr()?;
        let body = self.block()?;
        let end = body.span.end;
        Some(Expr {
            kind: ExprKind::While {
                condition: Box::new(condition),
                body,
            },
            span: Span::new(start, end),
        })
    }

    /// `"for" pattern "in" expr block` (§16). The iterable is a block-head
    /// position (§5.2, §16 `struct_literal` side condition).
    fn for_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::For)?.span.start;
        let pattern = self.pattern();
        self.expect_keyword(Keyword::In)?;
        let iterable = self.cond_expr()?;
        let body = self.block()?;
        let end = body.span.end;
        Some(Expr {
            kind: ExprKind::For {
                pattern: pattern?,
                iterable: Box::new(iterable),
                body,
            },
            span: Span::new(start, end),
        })
    }

    /// `"loop" block` (§16) — infinite loop, exits via `break` (no value).
    fn loop_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::Loop)?.span.start;
        let body = self.block()?;
        let end = body.span.end;
        Some(Expr {
            kind: ExprKind::Loop { body },
            span: Span::new(start, end),
        })
    }

    /// `match_expr = "match" expr "{" { match_arm } "}"` (§16). The scrutinee
    /// is a condition-like position: the block-head struct-literal restriction
    /// applies (§5.2).
    fn match_expr(&mut self) -> Option<Expr> {
        let start = self.expect_keyword(Keyword::Match)?.span.start;
        let scrutinee = self.cond_expr()?;
        self.expect(TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            match self.match_arm() {
                Some(arm) => arms.push(arm),
                None => self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]),
            }
            // Expression arms consume their own required trailing comma inside
            // `match_arm`; block arms take none, so the next arm (or `}`) may
            // follow directly. Eating a stray separator here keeps the loop
            // advancing after a failed arm.
            let _ = self.eat(TokenKind::Comma);
        }
        let end = self.expect(TokenKind::RBrace)?.span.end;
        Some(Expr {
            kind: ExprKind::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: Span::new(start, end),
        })
    }

    /// `match_arm = pattern [ "if" expr ] "=>" ( expr "," | block )` (§16).
    fn match_arm(&mut self) -> Option<MatchArm> {
        let pattern = self.pattern();
        let guard = if self.eat_keyword(Keyword::If).is_some() {
            Some(self.expr(0)?)
        } else {
            None
        };
        self.expect(TokenKind::FatArrow)?;
        let body = if self.at(TokenKind::LBrace) {
            let block = self.block()?;
            Expr {
                span: block.span,
                kind: ExprKind::Block(block),
            }
        } else {
            let body = self.expr(0)?;
            self.expect(TokenKind::Comma)?;
            body
        };
        Some(MatchArm {
            pattern: pattern?,
            guard,
            body,
        })
    }

    /// `pattern = ... | pattern "|" pattern` (§16) — the or-alternative is
    /// right-associative, matching the production's right-recursing
    /// derivation: `A | B | C` is `A | (B | C)`.
    fn pattern(&mut self) -> Option<Pattern> {
        let mut pattern = self.pattern_atom()?;
        if self.eat(TokenKind::Pipe).is_some() {
            let right = self.pattern()?;
            let span = pattern.span.join(right.span);
            pattern = Pattern {
                kind: PatternKind::Or {
                    left: Box::new(pattern),
                    right: Box::new(right),
                },
                span,
            };
        }
        Some(pattern)
    }

    /// The `pattern` production without the or-alternative: `_`, literals,
    /// bindings, paths, and the `(...)`, `path(...)`, and `path { ... }`
    /// destructuring forms (§16). A single bare identifier is a `Binding`;
    /// `path_expr` patterns are multi-segment paths (`Role::Admin`).
    fn pattern_atom(&mut self) -> Option<Pattern> {
        let token = *self.peek()?;
        match token.kind {
            TokenKind::Ident if self.text(&token) == "_" => {
                self.bump();
                Some(Pattern {
                    kind: PatternKind::Wildcard,
                    span: token.span,
                })
            }
            TokenKind::IntLit
            | TokenKind::FloatLit
            | TokenKind::StringLit
            | TokenKind::CharLit
            | TokenKind::Keyword(Keyword::True | Keyword::False) => {
                self.bump();
                Some(Pattern {
                    kind: PatternKind::Literal(self.literal_from_token(&token)),
                    span: token.span,
                })
            }
            TokenKind::Ident if self.text(&token) == "ref" => {
                // `[ "ref" ] IDENT` — `ref` prefixes a bare binding only.
                self.bump();
                let name = self.ident()?;
                let span = token.span.join(name.span);
                Some(Pattern {
                    kind: PatternKind::Binding {
                        name: name.name,
                        is_ref: true,
                    },
                    span,
                })
            }
            TokenKind::Ident | TokenKind::Keyword(Keyword::SelfType) => {
                // `path_expr = ( "Self" | IDENT ) { "::" IDENT }` (§16).
                let path = self.path()?;
                if self.eat(TokenKind::LParen).is_some() {
                    let args = self.patterns(TokenKind::RParen)?;
                    let end = self.expect(TokenKind::RParen)?.span.end;
                    Some(Pattern {
                        kind: PatternKind::Call { path, args },
                        span: Span::new(token.span.start, end),
                    })
                } else if self.at(TokenKind::LBrace) {
                    self.bump();
                    let (fields, rest) = self.field_pats()?;
                    let end = self.expect(TokenKind::RBrace)?.span.end;
                    Some(Pattern {
                        kind: PatternKind::Struct { path, fields, rest },
                        span: Span::new(token.span.start, end),
                    })
                } else if path.segments.len() == 1 && token.kind == TokenKind::Ident {
                    Some(Pattern {
                        kind: PatternKind::Binding {
                            name: path.segments[0].clone(),
                            is_ref: false,
                        },
                        span: path.span,
                    })
                } else {
                    let span = path.span;
                    Some(Pattern {
                        kind: PatternKind::Path(path),
                        span,
                    })
                }
            }
            TokenKind::LParen => {
                self.bump();
                let elems = self.patterns(TokenKind::RParen)?;
                let end = self.expect(TokenKind::RParen)?.span.end;
                Some(Pattern {
                    kind: PatternKind::Tuple(elems),
                    span: Span::new(token.span.start, end),
                })
            }
            _ => {
                self.error_here("expected pattern");
                None
            }
        }
    }

    /// `patterns = [ pattern { "," pattern } [ "," ] ]` (§16).
    fn patterns(&mut self, close: TokenKind) -> Option<Vec<Pattern>> {
        let mut patterns = Vec::new();
        while !self.at(close) && !self.eof() {
            patterns.push(self.pattern()?);
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        Some(patterns)
    }

    /// `field_pats = [ field_pat { "," field_pat } [ "," ] ]` with
    /// `field_pat = IDENT [ ":" pattern ]`; a trailing `".."` rest may follow
    /// the list (§16). Shorthand fields bind their own name.
    fn field_pats(&mut self) -> Option<(Vec<(String, Pattern)>, bool)> {
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::DotDot) && !self.eof() {
            let name = self.ident()?.name;
            let pattern = if self.eat(TokenKind::Colon).is_some() {
                self.pattern()?
            } else {
                let span = self.prev_span();
                Pattern {
                    kind: PatternKind::Binding {
                        name: name.clone(),
                        is_ref: false,
                    },
                    span,
                }
            };
            fields.push((name, pattern));
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        let rest = self.eat(TokenKind::DotDot).is_some();
        Some((fields, rest))
    }

    fn args(&mut self, close: TokenKind) -> Option<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.at(close) && !self.eof() {
            args.push(self.delimited_expr()?);
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        Some(args)
    }

    fn field_inits(&mut self) -> Option<Vec<(String, Option<Expr>)>> {
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.eof() {
            let name = self.ident()?.name;
            let value = if self.eat(TokenKind::Colon).is_some() {
                Some(self.expr(0)?)
            } else {
                None
            };
            fields.push((name, value));
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        Some(fields)
    }

    fn path(&mut self) -> Option<Path> {
        let first = self.path_segment()?;
        let start = first.span.start;
        let mut end = first.span.end;
        let mut segments = vec![first.name];
        while self.at(TokenKind::ColonColon) && self.can_start_path_segment_at(1) {
            self.bump();
            let segment = self.path_segment()?;
            end = segment.span.end;
            segments.push(segment.name);
        }
        Some(Path {
            segments,
            span: Span::new(start, end),
        })
    }

    fn path_segment(&mut self) -> Option<Ident> {
        match self.peek_kind()? {
            TokenKind::Ident
            | TokenKind::Keyword(Keyword::SelfType)
            | TokenKind::Keyword(Keyword::SelfValue) => {
                let token = self.bump();
                Some(Ident::new(self.text(&token), token.span))
            }
            _ => {
                self.error_here("expected path segment");
                None
            }
        }
    }

    fn ident(&mut self) -> Option<Ident> {
        let token = *self.peek()?;
        match token.kind {
            TokenKind::Ident => {
                self.bump();
                Some(Ident::new(self.text(&token), token.span))
            }
            _ => {
                self.error_here("expected identifier");
                None
            }
        }
    }

    fn maybe_generic_params(&mut self) {
        if self.at(TokenKind::Lt) {
            self.skip_balanced(TokenKind::Lt, TokenKind::Gt);
        }
    }

    fn type_args(&mut self) -> Vec<TypeArg> {
        if self.eat(TokenKind::Lt).is_none() {
            return Vec::new();
        }
        self.type_args_after_open()
    }

    fn turbofish(&mut self) -> Option<Vec<TypeArg>> {
        if !(self.at(TokenKind::ColonColon) && self.peek_kind_at(1) == Some(TokenKind::Lt)) {
            return None;
        }
        self.bump();
        self.bump();
        Some(self.type_args_after_open())
    }

    fn type_args_after_open(&mut self) -> Vec<TypeArg> {
        let mut args = Vec::new();
        while !self.at(TokenKind::Gt) && !self.eof() {
            // §16 `type_arg = type | IDENT "=" type`. Commit to the
            // associated-type binding by peeking `IDENT "="` — probing with
            // `ident()` and backtracking left its failure diagnostic in
            // `errors`, wrongly rejecting non-ident-headed type args like
            // `Vec<&str>` (issue #78).
            if self.at(TokenKind::Ident) && self.peek_kind_at(1) == Some(TokenKind::Eq) {
                if let Some(name) = self.ident()
                    && self.eat(TokenKind::Eq).is_some()
                    && let Some(ty) = self.ty()
                {
                    args.push(TypeArg::Assoc { name, ty });
                } else {
                    self.recover_until(&[TokenKind::Comma, TokenKind::Gt]);
                }
            } else if let Some(ty) = self.ty() {
                args.push(TypeArg::Type(ty));
            } else {
                self.recover_until(&[TokenKind::Comma, TokenKind::Gt]);
            }
            if self.eat(TokenKind::Comma).is_none() {
                break;
            }
        }
        let _ = self.expect(TokenKind::Gt);
        args
    }

    fn maybe_where_clause(&mut self) {
        if self.eat_ident_text("where").is_some() {
            self.recover_until(&[TokenKind::LBrace, TokenKind::Semi]);
        }
    }

    fn skip_item_body(&mut self) {
        if self.at(TokenKind::LBrace) {
            self.skip_balanced(TokenKind::LBrace, TokenKind::RBrace);
        } else {
            let _ = self.eat(TokenKind::Semi);
        }
    }

    fn skip_balanced(&mut self, open: TokenKind, close: TokenKind) {
        let mut depth = 0usize;
        while !self.eof() {
            if self.at(open) {
                depth += 1;
            } else if self.at(close) {
                if depth == 0 {
                    self.bump();
                    break;
                }
                depth -= 1;
                self.bump();
                if depth == 0 {
                    break;
                }
                continue;
            }
            self.bump();
        }
    }

    fn skip_doc_comments(&mut self) {
        while self.at(TokenKind::DocComment) {
            self.bump();
        }
    }

    fn infix_binding_power(&self) -> Option<(Infix, u8, u8)> {
        Some(match self.peek_kind()? {
            TokenKind::Eq => (Infix::Assign, 2, 1),
            TokenKind::PipeGt => (Infix::Op(BinaryOp::Pipe), 8, 9),
            TokenKind::Plus => (Infix::Op(BinaryOp::Add), 9, 10),
            TokenKind::Minus => (Infix::Op(BinaryOp::Sub), 9, 10),
            TokenKind::Star => (Infix::Op(BinaryOp::Mul), 10, 11),
            TokenKind::Slash => (Infix::Op(BinaryOp::Div), 10, 11),
            TokenKind::Percent => (Infix::Op(BinaryOp::Rem), 10, 11),
            TokenKind::EqEq => (Infix::Op(BinaryOp::Eq), 6, 7),
            TokenKind::Ne => (Infix::Op(BinaryOp::Ne), 6, 7),
            TokenKind::Lt => (Infix::Op(BinaryOp::Lt), 7, 8),
            TokenKind::Gt => (Infix::Op(BinaryOp::Gt), 7, 8),
            TokenKind::Le => (Infix::Op(BinaryOp::Le), 7, 8),
            TokenKind::Ge => (Infix::Op(BinaryOp::Ge), 7, 8),
            TokenKind::AmpAmp => (Infix::Op(BinaryOp::And), 5, 6),
            TokenKind::PipePipe => (Infix::Op(BinaryOp::Or), 4, 5),
            TokenKind::DotDot => (Infix::Op(BinaryOp::Range), 3, 4),
            TokenKind::DotDotEq => (Infix::Op(BinaryOp::RangeInclusive), 3, 4),
            _ => return None,
        })
    }

    fn recover_item(&mut self) {
        let start = self.pos;
        self.recover_until(&[TokenKind::Semi, TokenKind::RBrace]);
        if self.eat(TokenKind::Semi).is_some() || self.eat(TokenKind::RBrace).is_some() {
            return;
        }
        if self.pos == start && !self.eof() {
            self.bump();
        }
    }

    fn recover_until(&mut self, kinds: &[TokenKind]) {
        while !self.eof() && !kinds.iter().any(|kind| self.at(*kind)) {
            self.bump();
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        if self.at_keyword(keyword) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        self.eat_keyword(keyword).or_else(|| {
            self.error_here(format!("expected keyword `{keyword:?}`"));
            None
        })
    }

    fn at_ident_text(&self, text: &str) -> bool {
        self.peek()
            .is_some_and(|token| token.kind == TokenKind::Ident && token.text(self.source) == text)
    }

    fn eat_ident_text(&mut self, text: &str) -> Option<Token> {
        if self.at_ident_text(text) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        self.eat(kind).or_else(|| {
            self.error_here(format!("expected `{kind:?}`"));
            None
        })
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Keyword(found)) if found == keyword)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|token| token.kind)
    }

    fn peek_kind_at(&self, offset: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| token.kind)
    }

    fn can_start_path_segment_at(&self, offset: usize) -> bool {
        matches!(
            self.peek_kind_at(offset),
            Some(
                TokenKind::Ident
                    | TokenKind::Keyword(Keyword::SelfValue)
                    | TokenKind::Keyword(Keyword::SelfType)
            )
        )
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn prev_span(&self) -> Span {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map_or_else(Span::default, |token| token.span)
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos];
        self.pos += 1;
        token
    }

    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn error_here(&mut self, message: impl Into<String>) {
        let span = self
            .peek()
            .map_or_else(|| self.prev_span(), |token| token.span);
        self.error_at(span, message);
    }

    fn error_at(&mut self, span: Span, message: impl Into<String>) {
        self.errors.push(ParseError {
            message: message.into(),
            span,
        });
    }
}

/// Canonicalizes the overlapping `attr_arg` alternatives (§16): a bare
/// `IDENT`, `IDENT "=" expr`, and `IDENT "(" expr ")"` are all valid
/// expressions too, so they parse as expressions and the most specific attr
/// form is recovered from the shape afterwards.
fn classify_attr_expr(expr: Expr) -> AttrArg {
    let span = expr.span;
    match expr.kind {
        ExprKind::Path(path) if is_attr_ident(&path) => {
            let name = path.segments.into_iter().next().unwrap();
            AttrArg::Ident(Ident::new(name, path.span))
        }
        ExprKind::Assign { target, value } => match target.kind {
            ExprKind::Path(path) if is_attr_ident(&path) => {
                let name = path.segments.into_iter().next().unwrap();
                AttrArg::Assigned {
                    name: Ident::new(name, path.span),
                    value: *value,
                }
            }
            kind => AttrArg::Expr(Expr {
                kind: ExprKind::Assign {
                    target: Box::new(Expr {
                        kind,
                        span: target.span,
                    }),
                    value,
                },
                span,
            }),
        },
        ExprKind::Call {
            callee,
            type_args,
            mut args,
        } if type_args.is_empty() && args.len() == 1 && is_attr_ident_expr(&callee) => {
            let ExprKind::Path(path) = callee.kind else {
                unreachable!();
            };
            let name = path.segments.into_iter().next().unwrap();
            AttrArg::Call {
                name: Ident::new(name, path.span),
                arg: args.pop().unwrap(),
            }
        }
        kind => AttrArg::Expr(Expr { kind, span }),
    }
}

/// A single plain identifier — `self`/`Self` are keywords, not `IDENT` (§16.1).
fn is_attr_ident(path: &Path) -> bool {
    path.segments.len() == 1 && path.segments[0] != "self" && path.segments[0] != "Self"
}

fn is_attr_ident_expr(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Path(path) if is_attr_ident(path))
}

/// The §2.7 send-statement operand shape: `handle.method(args)` — a call whose
/// callee is a field access.
fn is_method_call(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::Call { callee, .. } if matches!(callee.kind, ExprKind::Field { .. })
    )
}

/// §16: `assign_target = IDENT | "self" "." IDENT | expr "." IDENT | "*" expr
/// | expr "[" expr "]"`. `self`/`Self` alone and multi-segment paths are not
/// assignable.
fn is_assign_target(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Path(path) => {
            path.segments.len() == 1 && path.segments[0] != "self" && path.segments[0] != "Self"
        }
        ExprKind::Field { .. } | ExprKind::Index { .. } => true,
        ExprKind::Unary { op, .. } => *op == UnaryOp::Deref,
        _ => false,
    }
}

/// Tokens that can begin an expression — must stay in sync with `prefix()`.
fn can_begin_expr(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::IntLit
            | TokenKind::FloatLit
            | TokenKind::StringLit
            | TokenKind::CharLit
            | TokenKind::Ident
            | TokenKind::Keyword(
                Keyword::True
                    | Keyword::False
                    | Keyword::SelfValue
                    | Keyword::SelfType
                    | Keyword::If
                    | Keyword::Match
                    | Keyword::While
                    | Keyword::For
                    | Keyword::Loop
            )
            | TokenKind::Bang
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Amp
            | TokenKind::LParen
            | TokenKind::LBrace
    )
}

fn is_primitive_type(text: &str) -> bool {
    matches!(
        text,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "u256"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "String"
            | "Address"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_function_item() {
        let program = parse_program("pub async fn add(a: i64, b: i64) -> i64 { a + b }").unwrap();
        assert_eq!(program.items.len(), 1);
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        assert!(func.is_async);
        assert_eq!(func.params.len(), 2);
    }

    #[test]
    fn parses_struct_with_nested_generic_field() {
        let program = parse_program("struct Bag { items: Map<String, Vec<u64>>, }").unwrap();
        let ItemKind::Struct(strukt) = &program.items[0].kind else {
            panic!("expected struct");
        };
        let Type::Path { args, .. } = &strukt.fields[0].ty else {
            panic!("expected generic path");
        };
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn doc_comments_and_attribute_arguments_do_not_break_items() {
        let source = r#"
            /// attaches to the next item
            @derive(Debug, Eq)
            struct User { id: u64 }
        "#;
        assert!(parse_program(source).is_ok());
    }

    #[test]
    fn attribute_arguments_are_preserved() {
        let source = "@derive(Debug, Eq)\nstruct User { id: u64 }";
        let program = parse_program(source).unwrap();
        let attr = &program.items[0].attrs[0];
        assert_eq!(attr.name.name, "derive");
        // Attribute span covers `@derive(Debug, Eq)`.
        assert_eq!(attr.span, Span::new(0, source.find('\n').unwrap()));
        let names: Vec<_> = attr
            .args
            .iter()
            .map(|arg| {
                let AttrArg::Ident(ident) = arg else {
                    panic!("expected bare ident arg, got {arg:?}");
                };
                ident.name.as_str()
            })
            .collect();
        assert_eq!(names, ["Debug", "Eq"]);
    }

    #[test]
    fn attribute_named_args_are_preserved() {
        let source = r#"
            @supervisor(strategy: "one_for_one", max_restarts: 5)
            struct Sup { x: i64 }
        "#;
        let program = parse_program(source).unwrap();
        let attr = &program.items[0].attrs[0];
        assert_eq!(attr.name.name, "supervisor");
        assert_eq!(attr.args.len(), 2);
        let AttrArg::Named { name, value } = &attr.args[0] else {
            panic!("expected named arg, got {:?}", attr.args[0]);
        };
        assert_eq!(name.name, "strategy");
        assert!(matches!(
            &value.kind,
            ExprKind::Literal(Literal::String(text)) if text == "\"one_for_one\""
        ));
        let AttrArg::Named { name, value } = &attr.args[1] else {
            panic!("expected named arg, got {:?}", attr.args[1]);
        };
        assert_eq!(name.name, "max_restarts");
        assert!(matches!(
            &value.kind,
            ExprKind::Literal(Literal::Int(text)) if text == "5"
        ));
    }

    #[test]
    fn attribute_assigned_call_and_expr_args_are_preserved() {
        let source = "@cfg(target = evm, feature(fast), CAP + 1)\nstruct S { x: i64 }";
        let program = parse_program(source).unwrap();
        let attr = &program.items[0].attrs[0];
        assert_eq!(attr.args.len(), 3);
        let AttrArg::Assigned { name, value } = &attr.args[0] else {
            panic!("expected assigned arg, got {:?}", attr.args[0]);
        };
        assert_eq!(name.name, "target");
        assert!(matches!(&value.kind, ExprKind::Path(path) if path.segments == ["evm"]));
        let AttrArg::Call { name, arg } = &attr.args[1] else {
            panic!("expected call arg, got {:?}", attr.args[1]);
        };
        assert_eq!(name.name, "feature");
        assert!(matches!(&arg.kind, ExprKind::Path(path) if path.segments == ["fast"]));
        let AttrArg::Expr(expr) = &attr.args[2] else {
            panic!("expected expr arg, got {:?}", attr.args[2]);
        };
        assert!(matches!(
            &expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn bare_attribute_has_no_args_and_name_span() {
        let program = parse_program("@test\nfn t() {}").unwrap();
        let attr = &program.items[0].attrs[0];
        assert_eq!(attr.name.name, "test");
        assert!(attr.args.is_empty());
        assert_eq!(attr.span, Span::new(0, 5));
    }

    #[test]
    fn actor_handler_attributes_are_preserved() {
        let source = r#"
            actor Worker {
                state: i64,
                @mailbox(capacity: 2048)
                pub fn run(&mut self, n: i64) {}
            }
        "#;
        let program = parse_program(source).unwrap();
        let ItemKind::Actor(actor) = &program.items[0].kind else {
            panic!("expected actor");
        };
        let handler = &actor.handlers[0];
        assert_eq!(handler.function.name.name, "run");
        assert_eq!(handler.attrs.len(), 1);
        let attr = &handler.attrs[0];
        assert_eq!(attr.name.name, "mailbox");
        // Span anchors to the `@` in the original source.
        assert_eq!(attr.span.start, source.find('@').unwrap());
        let AttrArg::Named { name, value } = &attr.args[0] else {
            panic!("expected named arg, got {:?}", attr.args[0]);
        };
        assert_eq!(name.name, "capacity");
        assert!(matches!(
            &value.kind,
            ExprKind::Literal(Literal::Int(text)) if text == "2048"
        ));
    }

    #[test]
    fn declaration_names_cannot_be_reserved_keywords() {
        let errors = parse_program("fn self() {}").unwrap_err();
        assert!(errors.iter().any(|err| err.message.contains("identifier")));
    }

    #[test]
    fn parses_turbofish_vec_let_mut_and_send() {
        let source = r#"
            fn main() {
                let mut a = parse::<i64>("42");
                let b = vec![1, 2, 3];
                let c = vec![0; 4];
                send worker.run(&mut a);
            }
        "#;
        assert!(parse_program(source).is_ok());
    }

    #[test]
    fn preserves_function_visibility_in_nested_items() {
        let source = r#"
            actor Counter {
                state: i64,
                pub fn inc(&mut self) {}
            }

            extern "C" {
                pub fn puts(s: &str);
            }
        "#;
        let program = parse_program(source).unwrap();
        let ItemKind::Actor(actor) = &program.items[0].kind else {
            panic!("expected actor");
        };
        assert_eq!(actor.handlers[0].function.visibility, Visibility::Public);
        let ItemKind::ExternBlock(extern_block) = &program.items[1].kind else {
            panic!("expected extern block");
        };
        assert_eq!(extern_block.functions[0].visibility, Visibility::Public);
    }

    #[test]
    fn if_condition_does_not_swallow_the_block_head() {
        // Regression: the then-block `{ ... }` must not be parsed as a struct
        // literal on the condition's trailing path operand.
        assert!(parse_program("fn f() -> i64 { if a < b { 1 } else { 2 } }").is_ok());
        assert!(parse_program("fn f() -> i64 { if flag { 1 } else { 2 } }").is_ok());
    }

    #[test]
    fn struct_literal_block_head_restriction() {
        // Outermost struct literal in a condition is rejected (§5.1)...
        assert!(parse_program("fn f() -> i64 { if x { f: 1 } { 1 } else { 2 } }").is_err());
        // ...parenthesized is accepted...
        assert!(parse_program("fn f() -> i64 { if (x { f: 1 }) == y { 1 } else { 2 } }").is_ok());
        // ...and a struct literal nested in call args (a delimited group) is fine.
        assert!(
            parse_program("fn f() -> i64 { if takes(Cfg { on: true }) { 1 } else { 2 } }").is_ok()
        );
        // The restriction must not leak into value position.
        assert!(parse_program("fn f() { let p = Point { x: 1, y: 2 }; }").is_ok());
    }

    #[test]
    fn block_head_restriction_does_not_leak_into_nested_if_blocks() {
        // A nested `if` in a non-parenthesized condition position must still
        // allow value-position struct literals inside its own blocks.
        let src = "fn f() -> i64 { if if a { Cfg { v: 1 }.on } else { false } { 1 } else { 2 } }";
        assert!(parse_program(src).is_ok());
    }

    #[test]
    fn parses_division_operator() {
        let program = parse_program("fn f(a: i64, b: i64) -> i64 { a / b }").unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        let block = func.body.as_ref().expect("function body");
        let tail = block.tail.as_ref().expect("tail expression");
        let ExprKind::Binary { op, .. } = &tail.kind else {
            panic!("expected binary expression");
        };
        assert_eq!(*op, BinaryOp::Div);
    }

    #[test]
    fn preserves_enum_variant_payloads() {
        let source = "enum Message { Quit, Move(i64, i64), Write { text: String }, }";
        let program = parse_program(source).unwrap();
        let ItemKind::Enum(enm) = &program.items[0].kind else {
            panic!("expected enum");
        };
        assert!(matches!(enm.variants[0].kind, VariantKind::Unit));
        let VariantKind::Tuple(types) = &enm.variants[1].kind else {
            panic!("expected tuple variant");
        };
        assert_eq!(types.len(), 2);
        let VariantKind::Struct(fields) = &enm.variants[2].kind else {
            panic!("expected struct variant");
        };
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn range_operators_cannot_chain() {
        for source in [
            "fn f() { let r = a..b..c; }",
            "fn f() { let r = a..=b..c; }",
            "fn f() { let r = a..b..=c; }",
        ] {
            let errors = parse_program(source).unwrap_err();
            // The diagnostic must anchor to the second range operator
            // (`..=` starts with `..`, so the second `..` match is its start).
            let second_op = source.match_indices("..").nth(1).unwrap().0;
            assert!(
                errors
                    .iter()
                    .any(|err| err.message.contains("chained") && err.span.start == second_op),
                "{source}: expected a range-chaining error at {second_op}, got {errors:?}"
            );
        }
    }

    #[test]
    fn send_operand_must_be_method_call() {
        // §2.7: inside a send-statement, anything but `handle.method(args)` is
        // a parse error.
        for source in [
            "fn f() { send 42; }",
            "fn f() { send free_fn(x); }",
            "fn f() { send (x); }",
            "fn f() { send vec![1]; }",
        ] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors.iter().any(|err| err.message.contains("method call")),
                "{source}: expected a method-call error, got {errors:?}"
            );
        }
    }

    #[test]
    fn send_method_call_operands_accepted() {
        let source = r#"
            fn f(worker: Handle<Worker>) {
                send worker.run(1);
                send hub.pool.run(2);
                send self.notify(3);
                send worker.push::<i64>(4);
            }
        "#;
        assert!(parse_program(source).is_ok());
    }

    #[test]
    fn send_head_before_non_expression_token_is_identifier() {
        // §2.7: the send-statement opens only when the next token can begin an
        // expression; otherwise `send` is an ordinary identifier.
        assert!(parse_program("fn f() { send = x; }").is_ok());
        assert!(parse_program("fn f() { send.reset(); }").is_ok());
    }

    #[test]
    fn assignment_targets_follow_grammar() {
        let source = r#"
            fn f(p: Point, arr: Vec<i64>, ptr: &mut i64) {
                x = 1;
                p.f = 2;
                arr[0] = 3;
                *ptr = 4;
                self.state = 5;
                a = b = c;
            }
        "#;
        assert!(parse_program(source).is_ok());
    }

    #[test]
    fn assignment_rejects_invalid_targets() {
        for source in [
            "fn f() { 1 + 2 = 3; }",
            "fn f() { g() = 1; }",
            "fn f() { a::b = 1; }",
            "fn f() { x? = 1; }",
            "fn f() { self = x; }",
        ] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|err| err.message.contains("assignment target")),
                "{source}: expected an assignment-target error, got {errors:?}"
            );
        }
    }

    #[test]
    fn single_and_parenthesized_ranges_parse() {
        assert!(parse_program("fn f() { let r = lo..hi; }").is_ok());
        assert!(parse_program("fn f() { let r = lo..=hi; }").is_ok());
        assert!(parse_program("fn f() { let r = (a..b)..c; }").is_ok());
    }

    #[test]
    fn offchain_and_async_apply_only_to_fn_items() {
        for (source, modifier) in [
            ("offchain struct S { x: i64 }", "offchain"),
            ("async struct S { x: i64 }", "async"),
            ("pub async struct S { x: i64 }", "async"),
            ("async trait T {}", "async"),
            ("offchain mod m;", "offchain"),
        ] {
            let errors = parse_program(source).unwrap_err();
            // The diagnostic must anchor to the misplaced modifier's own span.
            let start = source.find(modifier).unwrap();
            let span = Span::new(start, start + modifier.len());
            assert!(
                errors
                    .iter()
                    .any(|err| err.message.contains("applies only to `fn` items")
                        && err.span == span),
                "{source}: expected a modifier error at {span:?}, got {errors:?}"
            );
        }
        assert!(parse_program("pub offchain async fn f() {}").is_ok());
        assert!(parse_program("offchain fn g() {}").is_ok());
    }

    #[test]
    fn pub_rejected_where_grammar_omits_it() {
        for source in [
            "pub impl User {}",
            "pub actor A { state: i64, }",
            "pub onchain mod token {}",
            "pub extern \"C\" { fn f(); }",
        ] {
            let errors = parse_program(source).unwrap_err();
            // `pub` heads each source, so the diagnostic must anchor to 0..3.
            assert!(
                errors
                    .iter()
                    .any(|err| err.message.contains("`pub` is not allowed")
                        && err.span == Span::new(0, 3)),
                "{source}: expected a pub-placement error at 0..3, got {errors:?}"
            );
        }
        for source in [
            "pub struct S { x: i64 }",
            "pub enum E { A }",
            "pub trait T {}",
            "pub mod m;",
            "pub use crate::api;",
            "pub const X: i64 = 1;",
            "pub type Y = i64;",
        ] {
            assert!(
                parse_program(source).is_ok(),
                "{source}: `pub` should be accepted here"
            );
        }
    }

    #[test]
    fn vec_bang_requires_square_brackets() {
        for source in ["fn f() { let v = vec!(1); }", "fn f() { let v = vec!{1}; }"] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors.iter().any(|err| err.message.contains("vec!")),
                "{source}: expected a vec! error, got {errors:?}"
            );
        }
    }

    #[test]
    fn vec_without_bang_is_plain_identifier() {
        assert!(parse_program("fn f() { let v = vec; }").is_ok());
    }

    /// Parses `fn f() { let r = <expr>; }` and returns the bound expression.
    fn let_value(expr_src: &str) -> Expr {
        let source = format!("fn f() {{ let r = {expr_src}; }}");
        let program = parse_program(&source).unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        let Stmt::Let { value, .. } = &func.body.as_ref().unwrap().statements[0] else {
            panic!("expected let statement");
        };
        value.clone()
    }

    fn path_named(expr: &Expr, name: &str) -> bool {
        matches!(&expr.kind, ExprKind::Path(path) if path.segments == [name])
    }

    #[test]
    fn pipe_stage_question_wraps_accumulated_pipe() {
        // §5.7: `input |> parse?` is `(input |> parse)?`, never `input |> (parse?)`.
        let value = let_value("input |> parse?");
        let ExprKind::ErrorProp(inner) = &value.kind else {
            panic!("expected ErrorProp at the top, got {value:?}");
        };
        let ExprKind::Binary { op, left, right } = &inner.kind else {
            panic!("expected pipe binary inside ErrorProp");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        assert!(path_named(left, "input"));
        assert!(path_named(right, "parse"));
    }

    #[test]
    fn pipe_chain_question_on_each_stage() {
        // `a |> f? |> g?` nests as ErrorProp(Binary(ErrorProp(Binary(a, f)), g)).
        let value = let_value("a |> f? |> g?");
        let ExprKind::ErrorProp(outer) = &value.kind else {
            panic!("expected outer ErrorProp");
        };
        let ExprKind::Binary { op, left, right } = &outer.kind else {
            panic!("expected outer pipe binary");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        assert!(path_named(right, "g"));
        let ExprKind::ErrorProp(mid) = &left.kind else {
            panic!("expected inner ErrorProp");
        };
        let ExprKind::Binary { op, left, right } = &mid.kind else {
            panic!("expected inner pipe binary");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        assert!(path_named(left, "a"));
        assert!(path_named(right, "f"));
    }

    #[test]
    fn pipe_stage_with_args_keeps_question_on_pipe() {
        let value = let_value("x |> f(a)?");
        let ExprKind::ErrorProp(inner) = &value.kind else {
            panic!("expected ErrorProp at the top");
        };
        let ExprKind::Binary { op, right, .. } = &inner.kind else {
            panic!("expected pipe binary");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        let ExprKind::Call { callee, args, .. } = &right.kind else {
            panic!("expected call stage");
        };
        assert!(path_named(callee, "f"));
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn pipe_stage_method_chain_is_stage_callee() {
        // The field chain belongs to the stage, not to the pipe expression.
        let value = let_value("x |> svc.parse?");
        let ExprKind::ErrorProp(inner) = &value.kind else {
            panic!("expected ErrorProp at the top");
        };
        let ExprKind::Binary { op, right, .. } = &inner.kind else {
            panic!("expected pipe binary");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        let ExprKind::Field { base, name } = &right.kind else {
            panic!("expected field-chain stage");
        };
        assert_eq!(name, "parse");
        assert!(path_named(base, "svc"));
    }

    #[test]
    fn pipe_stage_turbofish_without_parens() {
        // `x |> parse::<i64>` desugars like `x |> parse::<i64>()`.
        let value = let_value("x |> parse::<i64>?");
        let ExprKind::ErrorProp(inner) = &value.kind else {
            panic!("expected ErrorProp at the top");
        };
        let ExprKind::Binary { right, .. } = &inner.kind else {
            panic!("expected pipe binary");
        };
        let ExprKind::Call {
            callee,
            type_args,
            args,
        } = &right.kind
        else {
            panic!("expected zero-arg call stage");
        };
        assert!(path_named(callee, "parse"));
        assert_eq!(type_args.len(), 1);
        assert!(args.is_empty());
    }

    #[test]
    fn pipe_stage_is_callee_not_arithmetic() {
        // A stage is only a callee: `x |> a + b` is `(x |> a) + b`.
        let value = let_value("x |> a + b");
        let ExprKind::Binary { op, left, right } = &value.kind else {
            panic!("expected `+` at the top");
        };
        assert_eq!(*op, BinaryOp::Add);
        assert!(path_named(right, "b"));
        let ExprKind::Binary { op, left, right } = &left.kind else {
            panic!("expected pipe binary on the left");
        };
        assert_eq!(*op, BinaryOp::Pipe);
        assert!(path_named(left, "x"));
        assert!(path_named(right, "a"));
    }

    #[test]
    fn pipe_rejects_non_callee_stage() {
        let errors = parse_program("fn f() { let r = x |> 42; }").unwrap_err();
        assert!(errors.iter().any(|err| err.message.contains("pipe stage")));
    }

    #[test]
    fn pipe_rejects_closure_stage_for_now() {
        let errors = parse_program("fn f() { let r = x |> (v); }").unwrap_err();
        assert!(
            errors
                .iter()
                .any(|err| err.message.contains("not yet implemented"))
        );
    }

    #[test]
    fn question_outside_pipe_unchanged() {
        let value = let_value("fetch()?");
        let ExprKind::ErrorProp(inner) = &value.kind else {
            panic!("expected ErrorProp");
        };
        assert!(matches!(&inner.kind, ExprKind::Call { .. }));
    }

    #[test]
    fn impl_trait_for_type_records_trait_ref() {
        let program = parse_program("impl Display for User {}").unwrap();
        let ItemKind::Impl(imp) = &program.items[0].kind else {
            panic!("expected impl");
        };
        let trait_ref = imp.trait_ref.as_ref().expect("trait ref recorded");
        assert_eq!(trait_ref.path.segments, ["Display"]);
        assert!(trait_ref.args.is_empty());
        assert!(matches!(
            &imp.target,
            Type::Path { path, .. } if path.segments == ["User"]
        ));
    }

    #[test]
    fn inherent_impl_has_no_trait_ref() {
        let program = parse_program("impl User {}").unwrap();
        let ItemKind::Impl(imp) = &program.items[0].kind else {
            panic!("expected impl");
        };
        assert!(imp.trait_ref.is_none());
    }

    #[test]
    fn impl_generic_trait_with_where_clause_parses() {
        let source = "impl<T> Convert<T> for Wrapper<T> where T: Clone {}";
        let program = parse_program(source).unwrap();
        let ItemKind::Impl(imp) = &program.items[0].kind else {
            panic!("expected impl");
        };
        let trait_ref = imp.trait_ref.as_ref().expect("trait ref recorded");
        assert_eq!(trait_ref.path.segments, ["Convert"]);
        assert_eq!(trait_ref.args.len(), 1);
    }

    #[test]
    fn impl_rejects_non_path_trait_before_for() {
        let errors = parse_program("impl &Foo for Bar {}").unwrap_err();
        assert!(errors.iter().any(|err| err.message.contains("trait path")));
    }

    #[test]
    fn trait_with_generics_supertraits_and_where_parses() {
        let source = r#"
            trait Convert<T> {}
            trait Loggable: Printable {}
            trait Bounded: Convert<i64> + Printable + 'static where Self: Printable {}
        "#;
        let program = parse_program(source).unwrap();
        let names: Vec<_> = program
            .items
            .iter()
            .map(|item| {
                let ItemKind::Trait(tr) = &item.kind else {
                    panic!("expected trait");
                };
                tr.name.name.as_str()
            })
            .collect();
        assert_eq!(names, ["Convert", "Loggable", "Bounded"]);
    }

    #[test]
    fn non_ident_headed_type_args_parse() {
        // Issue #78: the assoc-binding probe left a stale "expected
        // identifier" error behind, wrongly rejecting any type arg that does
        // not start with an identifier.
        for source in [
            "fn f(v: Vec<&str>) {}",
            "fn f(v: Vec<(i64, i64)>) {}",
            "fn f(v: Vec<[u8; 4]>) {}",
            "fn f(v: Map<String, Vec<&str>>) {}",
        ] {
            let result = parse_program(source);
            assert!(result.is_ok(), "{source}: {result:?}");
        }
        // The reference type arg survives structurally, not just error-free.
        let program = parse_program("fn f(v: Vec<&str>) {}").unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        let Param::Named { ty, .. } = &func.params[0] else {
            panic!("expected named param");
        };
        let Type::Path { args, .. } = ty else {
            panic!("expected generic path type");
        };
        let TypeArg::Type(Type::Reference { mutable, inner }) = &args[0] else {
            panic!("expected reference type arg, got {args:?}");
        };
        assert!(!mutable);
        assert!(matches!(inner.as_ref(), Type::Primitive(name) if name == "str"));
    }

    #[test]
    fn assoc_binding_with_non_ident_headed_value_parses() {
        // The binding side of `type_arg` still works when its value type is
        // itself non-ident-headed.
        let program = parse_program("fn f(it: &dyn Iter<Item = Vec<&str>>) {}").unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        let Param::Named {
            ty: Type::Reference { inner, .. },
            ..
        } = &func.params[0]
        else {
            panic!("expected reference param");
        };
        let Type::Dyn(trait_ref) = inner.as_ref() else {
            panic!("expected dyn type");
        };
        let TypeArg::Assoc { name, ty } = &trait_ref.args[0] else {
            panic!("expected assoc binding, got {:?}", trait_ref.args[0]);
        };
        assert_eq!(name.name, "Item");
        assert!(matches!(ty, Type::Path { .. }));
    }

    #[test]
    fn dyn_trait_type_args_with_assoc_binding() {
        let program = parse_program("fn f(it: &dyn Iter<Item = i64>) {}").unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        let Param::Named { ty, .. } = &func.params[0] else {
            panic!("expected named param");
        };
        let Type::Reference { inner, .. } = ty else {
            panic!("expected reference type");
        };
        let Type::Dyn(trait_ref) = inner.as_ref() else {
            panic!("expected dyn type");
        };
        assert_eq!(trait_ref.path.segments, ["Iter"]);
        assert!(matches!(&trait_ref.args[0], TypeArg::Assoc { name, .. } if name.name == "Item"));
    }

    /// Parses `fn f() -> i64 { <source> }` and returns the block's tail
    /// expression.
    fn tail_expr(source: &str) -> Expr {
        let source = format!("fn f() -> i64 {{ {source} }}");
        let program = parse_program(&source).unwrap();
        let ItemKind::Function(func) = &program.items[0].kind else {
            panic!("expected function");
        };
        func.body
            .as_ref()
            .expect("function body")
            .tail
            .as_ref()
            .expect("tail expression")
            .as_ref()
            .clone()
    }

    fn match_parts(source: &str) -> (Expr, Vec<MatchArm>) {
        let value = tail_expr(source);
        let ExprKind::Match { scrutinee, arms } = value.kind else {
            panic!("expected match expression, got {value:?}");
        };
        (*scrutinee, arms)
    }

    #[test]
    fn parses_match_guards_and_wildcard() {
        // §5.2: guards sit between the pattern and `=>`.
        let (scrutinee, arms) = match_parts(
            "match age { n if n < 13 => \"child\", n if n < 20 => \"teen\", n if n < 65 => \"adult\", _ => \"senior\", }",
        );
        assert!(path_named(&scrutinee, "age"));
        assert_eq!(arms.len(), 4);
        assert!(matches!(
            &arms[0].pattern.kind,
            PatternKind::Binding { name, is_ref: false } if name == "n"
        ));
        assert!(arms[0].guard.is_some());
        assert!(arms[3].guard.is_none());
        assert!(matches!(&arms[3].pattern.kind, PatternKind::Wildcard));
    }

    #[test]
    fn parses_match_tuple_destructuring() {
        // §5.2: tuple patterns with literal and binding elements. `(x)` is a
        // parenthesized pattern and `()` a unit tuple — degenerate `Tuple`s.
        let (_, arms) = match_parts(
            "match point { (0, 0) => \"origin\", (x, 0) => 1, (x, y) => 2, (z) => 3, () => 4, }",
        );
        let PatternKind::Tuple(elems) = &arms[0].pattern.kind else {
            panic!("expected tuple pattern");
        };
        assert!(matches!(
            &elems[0].kind,
            PatternKind::Literal(Literal::Int(text)) if text == "0"
        ));
        let PatternKind::Tuple(elems) = &arms[1].pattern.kind else {
            panic!("expected tuple pattern");
        };
        assert!(matches!(
            &elems[0].kind,
            PatternKind::Binding { name, .. } if name == "x"
        ));
        let PatternKind::Tuple(elems) = &arms[3].pattern.kind else {
            panic!("expected tuple pattern");
        };
        assert_eq!(elems.len(), 1);
        let PatternKind::Tuple(elems) = &arms[4].pattern.kind else {
            panic!("expected tuple pattern");
        };
        assert!(elems.is_empty());
    }

    #[test]
    fn parses_match_variant_call_patterns() {
        // §5.2: variant patterns, including qualified nesting like
        // `Err(AppError::Timeout { after })`.
        let (_, arms) = match_parts(
            "match result { Ok(user) => user, Err(AppError::NotFound) => 0, Err(AppError::Timeout { after }) => after, Err(e) => e, }",
        );
        let PatternKind::Call { path, args } = &arms[0].pattern.kind else {
            panic!("expected call pattern");
        };
        assert_eq!(path.segments, ["Ok"]);
        assert!(matches!(
            &args[0].kind,
            PatternKind::Binding { name, .. } if name == "user"
        ));
        let PatternKind::Call { path, args } = &arms[1].pattern.kind else {
            panic!("expected call pattern");
        };
        assert_eq!(path.segments, ["Err"]);
        assert!(matches!(
            &args[0].kind,
            PatternKind::Path(path) if path.segments == ["AppError", "NotFound"]
        ));
        let PatternKind::Call { args, .. } = &arms[2].pattern.kind else {
            panic!("expected call pattern");
        };
        let PatternKind::Struct { path, fields, rest } = &args[0].kind else {
            panic!("expected struct pattern");
        };
        assert_eq!(path.segments, ["AppError", "Timeout"]);
        assert_eq!(fields.len(), 1);
        assert!(matches!(
            &fields[0].1.kind,
            PatternKind::Binding { name, .. } if name == "after"
        ));
        assert!(!rest);
    }

    #[test]
    fn parses_struct_pattern_rest_and_or_patterns() {
        let (_, arms) = match_parts(
            "match role { Role::Editor { level } => 1, Role::Viewer | Role::Guest => 2, Role::Mod { name, .. } => 3, Role::None { .. } => 4, A | B | C => 5, }",
        );
        let PatternKind::Struct { fields, .. } = &arms[0].pattern.kind else {
            panic!("expected struct pattern");
        };
        assert_eq!(fields.len(), 1);
        let PatternKind::Or { left, right } = &arms[1].pattern.kind else {
            panic!("expected or-pattern");
        };
        assert!(matches!(
            &left.kind,
            PatternKind::Path(path) if path.segments == ["Role", "Viewer"]
        ));
        assert!(matches!(
            &right.kind,
            PatternKind::Path(path) if path.segments == ["Role", "Guest"]
        ));
        let PatternKind::Struct { fields, rest, .. } = &arms[2].pattern.kind else {
            panic!("expected struct pattern");
        };
        assert_eq!(fields.len(), 1);
        assert!(*rest);
        let PatternKind::Struct { fields, rest, .. } = &arms[3].pattern.kind else {
            panic!("expected struct pattern");
        };
        assert!(fields.is_empty());
        assert!(*rest);
        // `A | B | C` parses right-associatively: `A | (B | C)`. Single bare
        // identifiers are bindings, so `A` binds and `B | C` stays an `Or`.
        let PatternKind::Or { left, right } = &arms[4].pattern.kind else {
            panic!("expected or-pattern");
        };
        assert!(matches!(
            &left.kind,
            PatternKind::Binding { name, .. } if name == "A"
        ));
        assert!(matches!(&right.kind, PatternKind::Or { .. }));
    }

    #[test]
    fn parses_match_mixed_arm_bodies() {
        // §8.10.1: expression bodies carry the trailing comma, block bodies
        // none; `return` inside a block arm is a return_stmt.
        let (scrutinee, arms) = match_parts(
            "match worker.process(data) { Ok(result) => use_result(result), Err(ActorError::Dead) => { let fallback = default(); use_result(fallback) } Err(e) => { return Err(AppError::from(e)); } }",
        );
        assert!(matches!(&scrutinee.kind, ExprKind::Call { .. }));
        assert_eq!(arms.len(), 3);
        assert!(matches!(&arms[0].body.kind, ExprKind::Call { .. }));
        let ExprKind::Block(block) = &arms[1].body.kind else {
            panic!("expected block body");
        };
        assert_eq!(block.statements.len(), 1);
        assert!(block.tail.is_some());
        let ExprKind::Block(block) = &arms[2].body.kind else {
            panic!("expected block body");
        };
        assert!(matches!(block.statements[0], Stmt::Return(_)));
    }

    #[test]
    fn parses_match_ref_binding_and_wildcard_ident() {
        // `ref` borrows (§3.7); `_unused` is a regular identifier binding,
        // only bare `_` discards (§2.7).
        let (_, arms) =
            match_parts("match opt { Some(ref name) => name.len(), Some(_unused) => 0, _ => 1, }");
        let PatternKind::Call { args, .. } = &arms[0].pattern.kind else {
            panic!("expected call pattern");
        };
        assert!(matches!(
            &args[0].kind,
            PatternKind::Binding { name, is_ref: true } if name == "name"
        ));
        let PatternKind::Call { args, .. } = &arms[1].pattern.kind else {
            panic!("expected call pattern");
        };
        assert!(matches!(
            &args[0].kind,
            PatternKind::Binding { name, is_ref: false } if name == "_unused"
        ));
    }

    #[test]
    fn parses_nested_match_and_string_literal_pattern() {
        // §3.10: string literal patterns; a match arm body may itself be a
        // match (grammar nests without restriction).
        let source = "match kind { \"circle\" => match kind { \"unit\" => 1, _ => 2, }, _ => 3, }";
        let (_, arms) = match_parts(source);
        let PatternKind::Literal(Literal::String(text)) = &arms[0].pattern.kind else {
            panic!("expected string literal pattern");
        };
        assert_eq!(text, "\"circle\"");
        assert!(matches!(&arms[0].body.kind, ExprKind::Match { .. }));
    }

    #[test]
    fn match_scrutinee_struct_literal_restriction() {
        // §5.2: the scrutinee is a block-head position — an outermost struct
        // literal is rejected there, parenthesized is accepted, and nested
        // occurrences in delimited groups are fine.
        assert!(parse_program("fn f() -> bool { match x { on: true } { _ => true, } }").is_err());
        assert!(parse_program("fn f() -> bool { match (x { on: true }) { _ => true, } }").is_ok());
        assert!(
            parse_program("fn f() -> bool { match takes(Cfg { on: true }) { _ => true, } }")
                .is_ok()
        );
    }

    #[test]
    fn match_as_tail_and_non_tail_statement() {
        // A tail match needs no semicolon; a non-tail match statement is a
        // block-like expression and needs its `;`, like `if` and blocks.
        assert!(parse_program("fn f(x: i64) -> i64 { match x { _ => 1, } }").is_ok());
        // §16: `{ match_arm }` is zero-or-more; exhaustiveness is semantic.
        assert!(parse_program("fn f(x: i64) { match x {} }").is_ok());
        assert!(parse_program("fn f(x: i64) { match x { _ => 1, }; let y = 2; }").is_ok());
        let errors = parse_program("fn f(x: i64) { match x { _ => 1, } let y = 2; }").unwrap_err();
        assert!(
            errors
                .iter()
                .any(|err| err.message.contains("expected `RBrace`")),
            "expected a missing-terminator error, got {errors:?}"
        );
    }

    #[test]
    fn match_heads_pipe_and_pipe_in_arm_body() {
        // A match may head a pipe chain; an arm body may itself be a pipe
        // (§5.6), including the stage-trailing `?` (§5.7).
        let value = let_value("match kind { \"circle\" => 1, _ => 2, } |> double");
        let ExprKind::Binary {
            op: BinaryOp::Pipe, ..
        } = &value.kind
        else {
            panic!("expected pipe at the top, got {value:?}");
        };
        assert!(
            parse_program(
                "fn f(v: String) -> i64 { match v { Ok(s) => s |> parse::<i64>?, _ => 0, } }"
            )
            .is_ok()
        );
    }

    #[test]
    fn match_arm_grammar_violations() {
        for (source, needle) in [
            // Expression bodies require the trailing comma, even the last arm
            // (§16 `match_arm`).
            ("fn f() -> i64 { match x { _ => 1 } }", "expected `Comma`"),
            ("fn f() -> i64 { match x { _ 1 } }", "expected `FatArrow`"),
            // The pattern grammar's `literal` is unsigned — no `-1` patterns.
            ("fn f() { match x { -1 => 1, } }", "expected pattern"),
            // `[ "ref" ] IDENT` — `ref` prefixes a bare binding only.
            ("fn f() { match x { ref 1 => 2, } }", "expected identifier"),
            // `return` is a statement, not an `expr` alternative (§16), so it
            // cannot be an expression arm body; §5.2's own example shows this
            // form — spec debt tracked in #89.
            (
                "fn f() { match r { Err(e) => return Err(e), } }",
                "expected expression",
            ),
            // `field_pat = IDENT [ ":" pattern ]` has no `ref` shorthand;
            // §3.7's `Role::Editor { ref level }` example shows this form —
            // spec debt tracked in #89.
            (
                "fn f() { match r { Role::Editor { ref level } => 1, } }",
                "expected `RBrace`",
            ),
        ] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors.iter().any(|err| err.message.contains(needle)),
                "{source}: expected an error containing {needle:?}, got {errors:?}"
            );
        }
    }

    #[test]
    fn match_arm_error_recovery_does_not_cascade() {
        // A failed arm records one diagnostic and the following arm still
        // parses — recovery must not swallow the rest of the match.
        let errors =
            parse_program("fn f(x: i64) -> i64 { match x { 1 => , 2 => 3, } }").unwrap_err();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("expected expression"));
    }

    #[test]
    fn parses_while_and_while_let() {
        // §5.5 / §5.4 shapes.
        let value = tail_expr("while connection.is_alive() { handle(poll()); }");
        let ExprKind::While { condition, body } = value.kind else {
            panic!("expected while, got {value:?}");
        };
        assert!(matches!(&condition.kind, ExprKind::Call { .. }));
        assert_eq!(body.statements.len(), 1);

        let value = tail_expr("while let Ok(msg) = connection.read() { handle(msg); }");
        let ExprKind::WhileLet {
            pattern, scrutinee, ..
        } = value.kind
        else {
            panic!("expected while let, got {value:?}");
        };
        assert!(matches!(
            &pattern.kind,
            PatternKind::Call { path, .. } if path.segments == ["Ok"]
        ));
        assert!(matches!(&scrutinee.kind, ExprKind::Call { .. }));
    }

    #[test]
    fn parses_for_destructuring_and_ranges() {
        let value =
            tail_expr("for (index, value) in items.iter() |> enumerate() { print(index); }");
        let ExprKind::For {
            pattern, iterable, ..
        } = value.kind
        else {
            panic!("expected for, got {value:?}");
        };
        assert!(matches!(
            &pattern.kind,
            PatternKind::Tuple(elems) if elems.len() == 2
        ));
        // §5.6: a pipe chain is a valid iterable.
        assert!(matches!(
            &iterable.kind,
            ExprKind::Binary {
                op: BinaryOp::Pipe,
                ..
            }
        ));

        let value = tail_expr("for User { name, age, .. } in users { print(name); }");
        let ExprKind::For { pattern, .. } = value.kind else {
            panic!("expected for, got {value:?}");
        };
        let PatternKind::Struct { fields, rest, .. } = pattern.kind else {
            panic!("expected struct pattern");
        };
        assert_eq!(fields.len(), 2);
        assert!(rest);

        let value = tail_expr("for i in 0..10 { log(i); }");
        let ExprKind::For { iterable, .. } = value.kind else {
            panic!("expected for, got {value:?}");
        };
        assert!(matches!(
            &iterable.kind,
            ExprKind::Binary {
                op: BinaryOp::Range,
                ..
            }
        ));
    }

    #[test]
    fn parses_loop_with_break_and_continue() {
        // §5.5: infinite loop exits via break; a nested while coexists.
        // Non-tail block-like statements carry their `;` per `expr_stmt`
        // (loosening that is #62's call, not this parser's).
        let source = r#"
            loop {
                let event = poll();
                if event.is_shutdown() {
                    break;
                };
                if event.is_skip() {
                    continue;
                };
                while queue.has_work() {
                    process(queue.next());
                };
            }
        "#;
        let value = tail_expr(source);
        let ExprKind::Loop { body } = value.kind else {
            panic!("expected loop, got {value:?}");
        };
        assert_eq!(body.statements.len(), 4);
        let Stmt::Expr(Expr {
            kind: ExprKind::If { then_block, .. },
            ..
        }) = &body.statements[1]
        else {
            panic!("expected if statement");
        };
        assert!(matches!(then_block.statements[0], Stmt::Break));
        let Stmt::Expr(Expr {
            kind: ExprKind::If { then_block, .. },
            ..
        }) = &body.statements[2]
        else {
            panic!("expected if statement");
        };
        assert!(matches!(then_block.statements[0], Stmt::Continue));
        assert!(matches!(
            &body.statements[3],
            Stmt::Expr(Expr {
                kind: ExprKind::While { .. },
                ..
            })
        ));
    }

    #[test]
    fn parses_if_let_with_else_and_nesting() {
        // §5.4: single-pattern matching without a full match.
        let value = tail_expr(
            "if let Some(user) = find_user(42) { process(user); } else { log(\"miss\"); }",
        );
        let ExprKind::IfLet {
            pattern,
            scrutinee,
            then_block,
            else_block,
        } = value.kind
        else {
            panic!("expected if let, got {value:?}");
        };
        assert!(matches!(
            &pattern.kind,
            PatternKind::Call { path, .. } if path.segments == ["Some"]
        ));
        assert!(matches!(&scrutinee.kind, ExprKind::Call { .. }));
        assert_eq!(then_block.statements.len(), 1);
        assert!(else_block.is_some());

        let value = tail_expr("if let Role::Admin = user.role { grant_access(); }");
        let ExprKind::IfLet {
            pattern,
            else_block,
            ..
        } = value.kind
        else {
            panic!("expected if let, got {value:?}");
        };
        assert!(matches!(
            &pattern.kind,
            PatternKind::Path(path) if path.segments == ["Role", "Admin"]
        ));
        assert!(else_block.is_none());
    }

    #[test]
    fn else_chains_reach_if_let() {
        // §16: an if's else may chain into another if_expr or an if_let_expr.
        let source = "if a { 1 } else if b { 2 } else if let Some(x) = y { 3 } else { 4 }";
        let value = tail_expr(source);
        let ExprKind::If {
            else_branch: Some(outer_else),
            ..
        } = &value.kind
        else {
            panic!("expected if with else, got {value:?}");
        };
        let ExprKind::If {
            else_branch: Some(inner_else),
            ..
        } = &outer_else.kind
        else {
            panic!("expected else if, got {outer_else:?}");
        };
        assert!(
            matches!(&inner_else.kind, ExprKind::IfLet { .. }),
            "expected else if let, got {inner_else:?}"
        );
    }

    #[test]
    fn loop_condition_and_iterable_struct_literal_restriction() {
        // §5.1/§16: while conditions and for iterables are block-head
        // positions — outermost struct literals rejected, parenthesized ok.
        assert!(parse_program("fn f() -> bool { while x { on: true } { } }").is_err());
        assert!(parse_program("fn f() -> bool { while (x { on: true }) { } }").is_ok());
        assert!(parse_program("fn f() { for x in cfg { on: true } { } }").is_err());
        assert!(parse_program("fn f() { for x in (cfg { on: true }) { } }").is_ok());
    }

    #[test]
    fn let_scrutinee_struct_literal_restriction() {
        // The §16 side condition omits if-let/while-let scrutinees, but the
        // block-head ambiguity is identical (§5.1 rationale), so the parser
        // applies the restriction there too — spec gap tracked as #89 item 3.
        assert!(parse_program("fn f() -> bool { if let Some(y) = x { on: true } { } }").is_err());
        assert!(parse_program("fn f() -> bool { if let Some(y) = (x { on: true }) { } }").is_ok());
        assert!(parse_program("fn f() { while let Some(y) = x { on: true } { } }").is_err());
        assert!(parse_program("fn f() { while let Some(y) = (x { on: true }) { } }").is_ok());
    }

    #[test]
    fn break_and_continue_follow_stmt_grammar() {
        // `break_stmt = "break" ";"` — no value, no labels (§16, Appendix D).
        for source in [
            "fn f() { loop { break 1; } }",
            "fn f() { loop { continue 2; } }",
        ] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|err| err.message.contains("expected `Semi`")),
                "{source}: expected a no-value error, got {errors:?}"
            );
        }
        // break/continue are ordinary statements per §16 — the loop-context
        // check is semantic (no §18 code exists yet); the parser accepts them
        // anywhere a statement goes.
        assert!(parse_program("fn f() { break; }").is_ok());
        assert!(parse_program("fn f() { continue; }").is_ok());
    }

    #[test]
    fn if_let_else_is_block_only() {
        // §16: if_let_expr's else is `[ "else" block ]` — no `else if` chains
        // after an if let.
        let errors =
            parse_program("fn f() -> i64 { if let Some(x) = y { 1 } else if z { 2 } else { 3 } }")
                .unwrap_err();
        assert!(
            errors.iter().any(|err| err.message.contains("LBrace")),
            "expected a block-required error, got {errors:?}"
        );
    }

    #[test]
    fn loops_as_tail_and_non_tail_statements() {
        assert!(parse_program("fn f(x: i64) { while x < 3 { x = x + 1; } }").is_ok());
        assert!(parse_program("fn f() { loop { break; }; let y = 2; }").is_ok());
        assert!(parse_program("fn f(items: Vec<i64>) { for x in items { }; let y = 2; }").is_ok());
        let errors = parse_program("fn f() { loop { break; } let y = 2; }").unwrap_err();
        assert!(
            errors
                .iter()
                .any(|err| err.message.contains("expected `RBrace`")),
            "expected a missing-terminator error, got {errors:?}"
        );
    }

    #[test]
    fn loop_construct_grammar_violations() {
        for (source, needle) in [
            ("fn f(x: i64) { for x y { } }", "expected keyword `In`"),
            ("fn f() { while x < 3 }", "LBrace"),
            ("fn f(x: i64) { while let Some(x) x { } }", "Eq"),
            ("fn f() { loop }", "LBrace"),
        ] {
            let errors = parse_program(source).unwrap_err();
            assert!(
                errors.iter().any(|err| err.message.contains(needle)),
                "{source}: expected an error containing {needle:?}, got {errors:?}"
            );
        }
    }
}

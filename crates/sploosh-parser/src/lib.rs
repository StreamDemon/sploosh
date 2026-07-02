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
    Parser::new(tokens).parse_program()
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

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    /// When set, a `struct_literal` may not be the outermost expression — the
    /// block-head restriction (§5.1, §5.2; §16 `struct_literal` side condition).
    no_struct_literal: bool,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            no_struct_literal: false,
        }
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
        loop {
            if self.eat(TokenKind::At).is_none() {
                break;
            }
            if let Some(name) = self.ident() {
                if self.eat(TokenKind::LParen).is_some() {
                    self.skip_balanced_after_open(TokenKind::LParen, TokenKind::RParen);
                }
                attrs.push(Attribute { name });
            }
        }
        attrs
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
            let _attrs = self.attrs();
            let visibility = if self.eat_keyword(Keyword::Pub).is_some() {
                Visibility::Public
            } else {
                Visibility::Private
            };
            let is_async = self.eat_keyword(Keyword::Async).is_some();
            if self.at_keyword(Keyword::Fn) {
                handlers.push(self.function_after_mods(visibility, is_async, false, true)?);
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
            self.bump().lexeme
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
            } else if self.at_ident_text("send")
                && self
                    .peek_kind_at(1)
                    .is_some_and(|kind| can_begin_expr(&kind))
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
            let Some((op, left_bp, right_bp)) = self.infix_binding_power() else {
                break;
            };
            if left_bp < min_bp {
                break;
            }
            let op_text = self.bump().lexeme;
            if op == "|>" {
                // §16: the RHS of `|>` is a `pipe_stage`, not a precedence-climbed
                // expression.
                let stage = self.pipe_stage()?;
                let span = lhs.span.join(stage.span);
                lhs = Expr {
                    kind: ExprKind::Binary {
                        op: op_text,
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
            lhs = if op == "=" {
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
            } else {
                Expr {
                    kind: ExprKind::Binary {
                        op: op_text,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    },
                    span,
                }
            };
            // The precedence table marks `..`/`..=` non-associative: a range
            // operand may not itself be an unparenthesized range.
            if matches!(op, ".." | "..=")
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
        let token = self.peek()?.clone();
        match token.kind {
            TokenKind::IntLit | TokenKind::FloatLit | TokenKind::StringLit | TokenKind::CharLit => {
                self.bump();
                let lit = match token.kind {
                    TokenKind::IntLit => Literal::Int(token.lexeme),
                    TokenKind::FloatLit => Literal::Float(token.lexeme),
                    TokenKind::StringLit => Literal::String(token.lexeme),
                    TokenKind::CharLit => Literal::Char(token.lexeme),
                    _ => unreachable!(),
                };
                Some(Expr {
                    kind: ExprKind::Literal(lit),
                    span: token.span,
                })
            }
            TokenKind::Keyword(Keyword::True | Keyword::False) => {
                self.bump();
                Some(Expr {
                    kind: ExprKind::Literal(Literal::Bool(token.lexeme == "true")),
                    span: token.span,
                })
            }
            TokenKind::Ident if token.lexeme == "vec" => {
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
                        segments: vec![token.lexeme],
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
                let op = self.bump().lexeme;
                if op == "&" {
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
            TokenKind::Keyword(Keyword::If) => self.if_expr(),
            _ => {
                self.error_here("expected expression");
                None
            }
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
                self.if_expr()?
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

    fn args(&mut self, close: TokenKind) -> Option<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.at(close.clone()) && !self.eof() {
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
                Some(Ident::new(token.lexeme, token.span))
            }
            _ => {
                self.error_here("expected path segment");
                None
            }
        }
    }

    fn ident(&mut self) -> Option<Ident> {
        let token = self.peek()?.clone();
        match token.kind {
            TokenKind::Ident => {
                self.bump();
                Some(Ident::new(token.lexeme, token.span))
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
            let checkpoint = self.pos;
            if let Some(name) = self.ident()
                && self.eat(TokenKind::Eq).is_some()
                && let Some(ty) = self.ty()
            {
                args.push(TypeArg::Assoc { name, ty });
            } else {
                self.pos = checkpoint;
                if let Some(ty) = self.ty() {
                    args.push(TypeArg::Type(ty));
                } else {
                    self.recover_until(&[TokenKind::Comma, TokenKind::Gt]);
                }
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
            if self.at(open.clone()) {
                depth += 1;
            } else if self.at(close.clone()) {
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

    fn skip_balanced_after_open(&mut self, open: TokenKind, close: TokenKind) {
        let mut depth = 1usize;
        while !self.eof() {
            if self.at(open.clone()) {
                depth += 1;
            } else if self.at(close.clone()) {
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

    fn infix_binding_power(&self) -> Option<(&'static str, u8, u8)> {
        Some(match self.peek_kind()? {
            TokenKind::Eq => ("=", 2, 1),
            TokenKind::PipeGt => ("|>", 8, 9),
            TokenKind::Plus => ("+", 9, 10),
            TokenKind::Minus => ("-", 9, 10),
            TokenKind::Star => ("*", 10, 11),
            TokenKind::Slash => ("/", 10, 11),
            TokenKind::Percent => ("%", 10, 11),
            TokenKind::EqEq => ("==", 6, 7),
            TokenKind::Ne => ("!=", 6, 7),
            TokenKind::Lt => ("<", 7, 8),
            TokenKind::Gt => (">", 7, 8),
            TokenKind::Le => ("<=", 7, 8),
            TokenKind::Ge => (">=", 7, 8),
            TokenKind::AmpAmp => ("&&", 5, 6),
            TokenKind::PipePipe => ("||", 4, 5),
            TokenKind::DotDot => ("..", 3, 4),
            TokenKind::DotDotEq => ("..=", 3, 4),
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
        while !self.eof() && !kinds.iter().any(|kind| self.at(kind.clone())) {
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
            .is_some_and(|token| token.kind == TokenKind::Ident && token.lexeme == text)
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
        self.eat(kind.clone()).or_else(|| {
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
        self.peek().map(|token| token.kind.clone())
    }

    fn peek_kind_at(&self, offset: usize) -> Option<TokenKind> {
        self.tokens
            .get(self.pos + offset)
            .map(|token| token.kind.clone())
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
        let token = self.tokens[self.pos].clone();
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
        ExprKind::Unary { op, .. } => op == "*",
        _ => false,
    }
}

/// Tokens that can begin an expression — must stay in sync with `prefix()`.
fn can_begin_expr(kind: &TokenKind) -> bool {
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
        assert_eq!(actor.handlers[0].visibility, Visibility::Public);
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
        assert_eq!(op, "/");
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
        assert_eq!(op, "|>");
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
        assert_eq!(op, "|>");
        assert!(path_named(right, "g"));
        let ExprKind::ErrorProp(mid) = &left.kind else {
            panic!("expected inner ErrorProp");
        };
        let ExprKind::Binary { op, left, right } = &mid.kind else {
            panic!("expected inner pipe binary");
        };
        assert_eq!(op, "|>");
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
        assert_eq!(op, "|>");
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
        assert_eq!(op, "|>");
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
        assert_eq!(op, "+");
        assert!(path_named(right, "b"));
        let ExprKind::Binary { op, left, right } = &left.kind else {
            panic!("expected pipe binary on the left");
        };
        assert_eq!(op, "|>");
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
}

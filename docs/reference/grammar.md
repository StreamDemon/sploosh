# Grammar (EBNF)

> Complete formal grammar for the Sploosh language.

The authoritative grammar is in [LANGUAGE_SPEC.md](../spec-plans/LANGUAGE_SPEC.md) Section 16. This page will expand the EBNF with additional notes and railroad diagrams as the parser is implemented.

```ebnf
program        = { directives } { item } ;
item           = fn_def | struct_def | enum_def | trait_def
               | impl_block | mod_def | use_stmt | actor_def
               | onchain_mod | const_def | type_alias | extern_block ;

fn_def         = [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")"
                 [ "->" type ] block ;
params         = [ param { "," param } ] ;
param          = IDENT ":" type ;

struct_def     = [ attrs ] [ "pub" ] "struct" IDENT [ generics ] "{" fields "}" ;
enum_def       = [ attrs ] [ "pub" ] "enum" IDENT [ generics ] "{" variants "}" ;
trait_def      = [ "pub" ] "trait" IDENT [ generics ] [ ":" bounds ] "{" { trait_item } "}" ;
trait_item     = fn_def | "type" IDENT ";" ;
impl_block     = "impl" [ generics ] [ trait "for" ] type "{" { impl_item } "}" ;
impl_item      = fn_def | "type" IDENT "=" type ";" ;
actor_def      = [ attrs ] "actor" IDENT [ generics ] "{" { actor_item } "}" ;
mod_def        = [ "pub" ] "mod" IDENT ( ";" | "{" { item } "}" ) ;
use_stmt       = "use" path [ "::" "{" idents "}" ] ";" ;
onchain_mod    = "onchain" "mod" IDENT "{" { onchain_item } "}" ;
extern_block   = "extern" [ STRING ] "{" { extern_fn } "}" ;
extern_fn      = [ "pub" ] "fn" IDENT "(" params ")" [ "->" type ] ";" ;

type           = prim_type | IDENT [ generics ] | "&" [ "mut" ] type
               | "[" type ";" EXPR "]" | "[" type "]"
               | "(" [ type { "," type } ] ")" | "fn" "(" types ")" "->" type
               | "dyn" IDENT [ generics ] ;
type_suffix    = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" | "bool" | "char" | "str" ;

block          = "{" { statement } [ expr ] "}" ;
statement      = let_stmt | expr_stmt | return_stmt | emit_stmt ;
let_stmt       = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
const_def      = [ "pub" ] "const" IDENT ":" type "=" expr ";" ;
emit_stmt      = "emit" expr ";" ;

expr           = literal | IDENT | path_expr
               | expr "." IDENT | expr "(" args ")" | expr "[" expr "]"
               | expr BINOP expr | UNOP expr | expr "?" | expr "as" type
               | if_expr | if_let_expr | match_expr | select_expr | block | closure
               | expr "|>" expr
               | "spawn" expr | "spawn" "async" block | "send" expr | "recv" expr
               | expr ".await"
               | "for" pattern "in" expr block
               | "while" expr block | while_let_expr | "loop" block ;

select_expr    = "select" "{" { select_arm } "}" ;
select_arm     = pattern "=" expr "=>" expr "," ;

closure        = [ "move" ] "|" params "|" ( expr | block ) ;

pattern        = "_" | literal | IDENT | [ "ref" ] IDENT
               | IDENT "(" patterns ")" | IDENT "{" field_pats [ ".." ] "}"
               | "(" patterns ")" | pattern "|" pattern ;

attrs          = { "@" IDENT [ "(" attr_args ")" ] } ;
directives     = { "#" "[" IDENT [ "(" directive_args ")" ] "]" } ;
```

<!-- TODO: Add railroad diagrams and parser-specific notes as implementation proceeds -->

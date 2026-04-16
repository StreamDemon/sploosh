# Grammar (EBNF)

> Complete formal grammar for the Sploosh language.

The authoritative grammar is in [LANGUAGE_SPEC.md](../spec-plans/LANGUAGE_SPEC.md) Section 16. This page mirrors the EBNF with the same production rules. The syntactic productions in §16 and the lexical productions in §16.1 together form the full grammar of Sploosh.

## Syntactic Productions

```ebnf
program        = { item } ;
item           = [ directives ] item_kind ;
item_kind      = fn_def | struct_def | enum_def | trait_def
               | impl_block | mod_def | use_stmt | actor_def
               | onchain_mod | const_def | type_alias | extern_block ;

fn_def         = [ attrs ] [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")"
                 [ "->" type ] block ;
params         = [ param { "," param } ] ;
param          = IDENT ":" type ;

struct_def     = [ attrs ] [ "pub" ] "struct" IDENT [ generics ] "{" fields "}" ;
fields         = field { "," field } [ "," ] ;
field          = [ "pub" ] IDENT ":" type ;

enum_def       = [ attrs ] [ "pub" ] "enum" IDENT [ generics ] "{" variants "}" ;
variants       = variant { "," variant } [ "," ] ;
variant        = IDENT [ "(" types ")" | "{" fields "}" ] ;

trait_def      = [ "pub" ] "trait" IDENT [ generics ] [ ":" bounds ] "{" { trait_item } "}" ;
trait_item     = fn_sig ( block | ";" ) | "type" IDENT [ ":" bounds ] ";" ;

impl_block     = "impl" [ generics ] [ trait_ref "for" ] type "{" { impl_item } "}" ;
impl_item      = fn_def | "type" IDENT "=" type ";" ;

actor_def      = [ attrs ] "actor" IDENT [ generics ] "{" { actor_item } "}" ;
actor_item     = field_def | fn_def ;

mod_def        = [ "pub" ] "mod" IDENT ( ";" | "{" { item } "}" ) ;
use_stmt       = "use" path [ "::" "{" idents "}" ] ";" ;

onchain_mod    = "onchain" "mod" IDENT "{" { onchain_item } "}" ;
onchain_item   = storage_block | fn_def | event_def ;
storage_block  = "storage" "{" fields "}" ;

extern_block   = "extern" STRING_LIT "{" { extern_fn } "}" ;
extern_fn      = "fn" IDENT "(" params ")" [ "->" type ] ";" ;

type           = prim_type | IDENT [ generics ] | "&" [ lifetime ] [ "mut" ] type
               | "[" type ";" expr "]" | "[" type "]"
               | "(" [ type { "," type } ] ")" | "fn" "(" types ")" "->" type
               | "dyn" IDENT [ generics ] ;
prim_type      = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" | "bool" | "char" | "str" | "String"
               | "Address" | "()" ;
types          = [ type { "," type } [ "," ] ] ;
type_alias     = [ "pub" ] "type" IDENT [ generics ] "=" type ";" ;
trait_ref      = IDENT [ generics ] ;
generics       = "<" type_params ">" ;
type_params    = type_param { "," type_param } ;
type_param     = IDENT [ ":" bounds ] | lifetime ;
bounds         = bound { "+" bound } ;
bound          = IDENT [ generics ] | lifetime ;

block          = "{" { statement } [ expr ] "}" ;
statement      = let_stmt | expr_stmt | return_stmt | emit_stmt ;
let_stmt       = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
const_def      = [ "pub" ] "const" IDENT ":" type "=" expr ";" ;
return_stmt    = "return" [ expr ] ";" ;
emit_stmt      = "emit" IDENT "{" field_inits "}" ";" ;
expr_stmt      = expr ";" ;

expr           = literal | IDENT | path_expr
               | expr "." IDENT | expr "(" args ")"  | expr "[" expr "]"
               | expr BINOP expr | UNOP expr | expr "?" | expr "as" type
               | if_expr | if_let_expr | match_expr | block | closure
               | expr "|>" expr
               | "spawn" expr | "spawn" "async" block
               | "send" expr | "recv" expr
               | expr ".await"
               | select_expr
               | "for" pattern "in" expr block
               | "while" expr block | while_let_expr | "loop" block ;

if_expr        = "if" expr block [ "else" ( if_expr | if_let_expr | block ) ] ;
if_let_expr    = "if" "let" pattern "=" expr block [ "else" block ] ;
while_let_expr = "while" "let" pattern "=" expr block ;
match_expr     = "match" expr "{" { match_arm } "}" ;
match_arm      = pattern [ "if" expr ] "=>" ( expr "," | block ) ;
select_expr    = "select" "{" { select_arm } "}" ;
select_arm     = pattern "=" expr "=>" ( expr "," | block ) ;
closure        = [ "move" ] "|" params "|" ( expr | block ) ;

path_expr      = IDENT { "::" IDENT } ;
path           = [ "crate" | "super" | "self" ] { "::" IDENT } ;
args           = [ expr { "," expr } [ "," ] ] ;

BINOP          = "+" | "-" | "*" | "/" | "%"
               | "==" | "!=" | "<" | ">" | "<=" | ">="
               | "&&" | "||"
               | ".." | "..=" ;
UNOP           = "!" | "-" ;

pattern        = "_" | literal | IDENT | [ "ref" ] IDENT
               | IDENT "(" patterns ")" | IDENT "{" field_pats [ ".." ] "}"
               | "(" patterns ")" | pattern "|" pattern ;
patterns       = [ pattern { "," pattern } [ "," ] ] ;
field_pats     = [ field_pat { "," field_pat } [ "," ] ] ;
field_pat      = IDENT [ ":" pattern ] ;
field_inits    = [ field_init { "," field_init } [ "," ] ] ;
field_init     = IDENT [ ":" expr ] ;
idents         = IDENT { "," IDENT } ;

fn_sig         = [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")" [ "->" type ] ;
field_def      = [ "pub" ] IDENT ":" type ;
event_def      = [ attrs ] "enum" IDENT "{" variants "}" ;

literal        = INT_LIT [ type_suffix ] | FLOAT_LIT [ type_suffix ]
               | STRING_LIT | CHAR_LIT
               | "true" | "false" ;
type_suffix    = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" ;

attrs          = { "@" IDENT [ "(" attr_args ")" ] } ;
attr_args      = attr_arg { "," attr_arg } ;
attr_arg       = IDENT [ ":" expr | "=" expr | "(" expr ")" ] | expr ;
directives     = { "#[" IDENT [ "(" dir_args ")" ] "]" } ;
dir_args       = attr_args ;
```

## Lexical Productions

```ebnf
(* Identifiers *)
IDENT          = ASCII_ALPHA_US { ASCII_ALNUM_US } ;
ASCII_ALPHA_US = "A" ... "Z" | "a" ... "z" | "_" ;
ASCII_ALNUM_US = ASCII_ALPHA_US | DIGIT ;

(* Keywords take precedence over IDENT — see LANGUAGE_SPEC.md §2.3 and §2.7. *)

(* Lifetime annotations *)
lifetime       = "'" IDENT ;

(* Integer literals *)
INT_LIT        = dec_lit | hex_lit | oct_lit | bin_lit ;
dec_lit        = DIGIT { DIGIT | "_" } ;
hex_lit        = "0x" HEX_DIGIT { HEX_DIGIT | "_" } ;
oct_lit        = "0o" OCT_DIGIT { OCT_DIGIT | "_" } ;
bin_lit        = "0b" BIN_DIGIT { BIN_DIGIT | "_" } ;

(* Float literals *)
FLOAT_LIT      = dec_lit "." dec_lit [ exp_part ]
               | dec_lit exp_part ;
exp_part       = ( "e" | "E" ) [ "+" | "-" ] dec_lit ;

(* String and character literals *)
STRING_LIT     = '"' { str_body_char } '"' ;
str_body_char  = UNICODE_SCALAR_EXCEPT_BACKSLASH_QUOTE | escape
               | "\" NEWLINE WHITESPACE ;   (* line continuation *)
CHAR_LIT       = "'" ( UNICODE_SCALAR_EXCEPT_BACKSLASH_APOS | escape ) "'" ;

escape         = "\" ( simple_escape | hex_escape | unicode_escape ) ;
simple_escape  = "n" | "r" | "t" | "\" | '"' | "'" | "0" ;
hex_escape     = "x" HEX_DIGIT HEX_DIGIT ;            (* value must be 0x00..0x7F *)
unicode_escape = "u" "{" HEX_DIGIT { HEX_DIGIT } "}" ; (* 1..6 hex digits, must be a valid Unicode scalar *)

(* Digit classes *)
DIGIT          = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
HEX_DIGIT      = DIGIT | "a" ... "f" | "A" ... "F" ;
OCT_DIGIT      = "0" ... "7" ;
BIN_DIGIT      = "0" | "1" ;
```

**Lexical constraints enforced by the lexer beyond the EBNF above:**

- Underscores in numeric literals must appear between two digits — leading, trailing, and consecutive underscores are a compile error.
- `hex_escape` values must be in the range `0x00`–`0x7F` (ASCII only). Use `unicode_escape` for values ≥ `0x80`.
- `unicode_escape` values must be a valid Unicode scalar value — surrogate code points `0xD800`–`0xDFFF` are rejected, as are values above `0x10FFFF`.
- Literal overflow (the integer value does not fit in its declared or inferred numeric type) is a compile error at parse time, not a runtime check.
- `CHAR_LIT` contains exactly one Unicode scalar value. Empty character literals and multi-character character literals are compile errors.

See LANGUAGE_SPEC.md §2.6 for worked examples of each literal form and §2.7 for the identifier rules in prose.

# Grammar (EBNF)

> Complete formal grammar for the Sploosh language.

The authoritative grammar is in [LANGUAGE_SPEC.md](../spec-plans/LANGUAGE_SPEC.md) Section 16. This page mirrors the EBNF with the same production rules. The syntactic productions in §16 and the lexical productions in §16.1 together form the full grammar of Sploosh.

## Syntactic Productions

```ebnf
program        = { item } ;
item           = [ directives ] item_kind ;
item_kind      = fn_def | struct_def | enum_def | trait_def
               | impl_block | mod_def | use_stmt | actor_def
               | onchain_mod | event_def | const_def | type_alias | extern_block ;

fn_def         = [ attrs ] [ "pub" ] [ "offchain" ] [ "async" ] "fn" IDENT [ generic_params ]
                 "(" params ")" [ "->" type ] [ where_clause ] block ;
params         = [ ( receiver | param ) { "," param } ] ;
receiver       = [ "&" [ "mut" ] ] "self" ;
param          = IDENT ":" type ;

struct_def     = [ attrs ] [ "pub" ] "struct" IDENT [ generic_params ] [ where_clause ]
                 "{" fields "}" ;
fields         = field { "," field } [ "," ] ;
field          = [ directives ] [ "pub" ] IDENT ":" type ;

enum_def       = [ attrs ] [ "pub" ] "enum" IDENT [ generic_params ] [ where_clause ]
                 "{" variants "}" ;
variants       = variant { "," variant } [ "," ] ;
variant        = [ directives ] IDENT [ "(" types ")" | "{" fields "}" ] ;

trait_def      = [ "pub" ] "trait" IDENT [ generic_params ] [ ":" bounds ]
                 [ where_clause ] "{" { trait_item } "}" ;
trait_item     = fn_sig ( block | ";" ) | "type" IDENT [ ":" bounds ] ";" ;

impl_block     = "impl" [ generic_params ] [ trait_ref "for" ] type [ where_clause ]
                 "{" { impl_item } "}" ;
impl_item      = fn_def | "type" IDENT "=" type ";" ;

actor_def      = [ attrs ] "actor" IDENT [ generic_params ] [ where_clause ]
                 "{" [ fields ] { fn_def } "}" ;
                 (* state fields first, comma-separated with optional trailing
                    comma exactly as struct fields; handler fns follow *)

mod_def        = [ "pub" ] "mod" IDENT ( ";" | "{" { item } "}" ) ;
use_stmt       = [ "pub" ] "use" path [ "::" "{" idents "}" ] ";" ;

onchain_mod    = "onchain" "mod" IDENT "{" { onchain_item } "}" ;
onchain_item   = [ directives ] ( storage_block | fn_def | onchain_event_def ) ;
storage_block  = "storage" "{" fields "}" ;

extern_block   = "extern" extern_target "{" { extern_fn } "}" ;
extern_target  = STRING_LIT [ "async" ] | "onchain" "mod" IDENT ;
extern_fn      = [ "pub" ] "fn" IDENT "(" params ")" [ "->" type ] ";" ;
                 (* async applies only to string-target extern blocks:
                    extern "C" async { ... }. extern onchain mod never accepts
                    async. *)

type           = prim_type | type_path [ type_args ] | "&" [ lifetime ] [ "mut" ] type
               | "[" type ";" expr "]" | "[" type "]"
               | "(" [ type { "," type } ] ")" | "fn" "(" types ")" "->" type
               | "dyn" trait_ref ;
prim_type      = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" | "bool" | "char" | "str" | "String"
               | "Address" | "()" ;
types          = [ type { "," type } [ "," ] ] ;
type_alias     = [ "pub" ] "type" IDENT [ generic_params ] "=" type ";" ;
type_path      = ( "Self" | IDENT ) { "::" IDENT } ;
trait_ref      = type_path [ type_args ] ;
generic_params = "<" generic_param { "," generic_param } [ "," ] ">" ;
generic_param  = IDENT [ ":" bounds ] | lifetime [ ":" lifetime_bounds ] ;
type_args      = "<" type_arg { "," type_arg } [ "," ] ">" ;
type_arg       = type | assoc_type_binding ;
assoc_type_binding = IDENT "=" type ;
bounds         = bound { "+" bound } ;
bound          = trait_ref | lifetime ;
where_clause   = "where" where_pred { "," where_pred } [ "," ] ;
where_pred     = type_path ":" bounds | lifetime ":" lifetime_bounds ;
lifetime_bounds = lifetime { "+" lifetime } ;

block          = "{" { statement } [ expr ] "}" ;
statement      = let_stmt | expr_stmt | return_stmt | emit_stmt
               | send_stmt | break_stmt | continue_stmt ;
let_stmt       = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
const_def      = [ "pub" ] "const" IDENT ":" type "=" expr ";" ;
return_stmt    = "return" [ expr ] ";" ;
emit_stmt      = "emit" IDENT "{" field_inits "}" ";" ;
send_stmt      = "send" expr ";" ;
                 (* "send" is contextual (§2.3.2, §2.7); expr must be a
                    method call on a handle — §8.2 *)
break_stmt     = "break" ";" ;
continue_stmt  = "continue" ";" ;
expr_stmt      = expr ";" ;

expr           = literal | "self" | IDENT | path_expr | struct_literal
               | expr "." IDENT [ turbofish ] | expr [ turbofish ] "(" args ")"
               | expr "[" expr "]" | assign_expr
               | expr BINOP expr | UNOP expr | "&" [ "mut" ] expr
               | expr "?" | expr "as" type
               | if_expr | if_let_expr | match_expr | block | closure
               | pipe_expr | vec_literal | assert_matches_expr
               | "spawn" expr | "spawn" "async" block
               | expr ".await"
               | select_expr
               | "for" pattern "in" expr block
               | "while" expr block | while_let_expr | "loop" block ;
assign_expr    = assign_target "=" expr ;
assign_target  = IDENT | "self" "." IDENT | expr "." IDENT | "*" expr
               | expr "[" expr "]" ;
                 (* assignment is right-associative (§2.4); the parser accepts
                    only assign_target on the left side. *)

struct_literal = path_expr "{" field_inits "}" ;
                 (* side condition (block-head restriction, Rust precedent):
                    a struct_literal may not appear as the outermost
                    expression of an "if"/"while" condition, a "match"
                    scrutinee, or a "for" iterable — parenthesize to use one
                    in those positions ("if (x { f: 1 }) == y { ... }").
                    See §5.1, §5.2. field_inits supports shorthand (§16
                    field_init). *)

pipe_expr      = expr "|>" pipe_stage { "|>" pipe_stage } ;
pipe_stage     = stage_callee [ "(" args ")" ] [ "?" ] ;
                 (* a stage's trailing "?" applies to the accumulated pipe
                    application result — see §5.7 and the §2.4 footnote *)
stage_callee   = path_expr [ turbofish ] { "." IDENT [ turbofish ] } | "(" closure ")" ;
turbofish      = "::" type_args ;

if_expr        = "if" expr block [ "else" ( if_expr | if_let_expr | block ) ] ;
if_let_expr    = "if" "let" pattern "=" expr block [ "else" block ] ;
while_let_expr = "while" "let" pattern "=" expr block ;
match_expr     = "match" expr "{" { match_arm } "}" ;
match_arm      = pattern [ "if" expr ] "=>" ( expr "," | block ) ;
select_expr    = "select" "{" { select_arm } "}" ;
select_arm     = pattern "=" expr "=>" ( expr "," | block ) ;
closure        = [ "move" ] "|" closure_params "|" ( expr | block ) ;
closure_params = [ closure_param { "," closure_param } [ "," ] ] ;
closure_param  = pattern [ ":" type ] ;

vec_literal    = "vec" "!" "[" ( args | expr ";" expr ) "]" ;
assert_matches_expr = "assert_matches" "(" expr "," pattern ")" ;

path_expr      = ( "Self" | IDENT ) { "::" IDENT } ;
path           = ( "crate" | "super" | "self" | IDENT ) { "::" IDENT } ;
args           = [ expr { "," expr } [ "," ] ] ;

BINOP          = "+" | "-" | "*" | "/" | "%"
               | "==" | "!=" | "<" | ">" | "<=" | ">="
               | "&&" | "||"
               | ".." | "..=" ;
UNOP           = "!" | "-" | "*" ;

pattern        = "_" | literal | [ "ref" ] IDENT | path_expr
               | path_expr "(" patterns ")" | path_expr "{" field_pats [ ".." ] "}"
               | "(" patterns ")" | pattern "|" pattern ;
patterns       = [ pattern { "," pattern } [ "," ] ] ;
field_pats     = [ field_pat { "," field_pat } [ "," ] ] ;
field_pat      = IDENT [ ":" pattern ] ;
field_inits    = [ field_init { "," field_init } [ "," ] ] ;
field_init     = IDENT [ ":" expr ] ;
idents         = IDENT { "," IDENT } ;

fn_sig         = [ "pub" ] [ "async" ] "fn" IDENT [ generic_params ]
                 "(" params ")" [ "->" type ] [ where_clause ] ;
event_def      = [ attrs ] "onchain" "enum" IDENT "{" variants "}" ;
onchain_event_def = [ attrs ] [ "onchain" ] "enum" IDENT "{" variants "}" ;
                 (* top-level event declarations require "onchain enum"
                    (§11.5). Inside onchain_mod, plain "enum Event" remains
                    valid because the context already marks it on-chain. *)

literal        = INT_LIT | FLOAT_LIT | STRING_LIT | CHAR_LIT
               | "true" | "false" ;

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

(* Reserved keywords (§2.3.1) take precedence over IDENT. Contextual
   keywords (§2.3.2) match IDENT everywhere outside their defined keyword
   positions — see §2.7 for the disambiguation mechanism. *)

(* Lifetime annotations *)
lifetime       = "'" IDENT ;

(* Integer literals *)
INT_LIT        = ( dec_lit | hex_lit | oct_lit | bin_lit ) [ int_suffix ] ;
dec_lit        = DIGIT { DIGIT | "_" } ;
hex_lit        = "0x" HEX_DIGIT { HEX_DIGIT | "_" } ;
oct_lit        = "0o" OCT_DIGIT { OCT_DIGIT | "_" } ;
bin_lit        = "0b" BIN_DIGIT { BIN_DIGIT | "_" } ;

(* Float literals *)
FLOAT_LIT      = ( dec_lit "." dec_lit [ exp_part ]
                 | dec_lit exp_part ) [ float_suffix ]
               | dec_lit float_suffix ;
exp_part       = ( "e" | "E" ) [ "+" | "-" ] dec_lit ;
int_suffix     = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256" ;
float_suffix   = "f32" | "f64" ;

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
- Numeric suffixes are part of the literal token and require no intervening whitespace. `42u32` is a suffixed `INT_LIT`; `42 u32` tokenizes as `INT_LIT` followed by `IDENT`.
- `CHAR_LIT` contains exactly one Unicode scalar value. Empty character literals and multi-character character literals are compile errors.

See LANGUAGE_SPEC.md §2.6 for worked examples of each literal form and §2.7 for the identifier rules in prose.

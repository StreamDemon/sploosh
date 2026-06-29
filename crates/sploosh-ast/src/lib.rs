//! Shared syntax data structures for the Sploosh compiler bootstrap.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn join(self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Actor(Actor),
    Module(Module),
    Use(Use),
    Const(Const),
    TypeAlias(TypeAlias),
    Trait(Trait),
    Impl(ImplBlock),
    OnchainModule(OnchainModule),
    ExternBlock(ExternBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Ident,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_offchain: bool,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Receiver {
        mutable: bool,
        by_ref: bool,
        span: Span,
    },
    Named {
        name: Ident,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub onchain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: Ident,
    pub kind: VariantKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantKind {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub handlers: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: Ident,
    pub items: Vec<Item>,
    pub inline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub target: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnchainModule {
    pub name: Ident,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    pub target: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(String),
    Path { path: Path, args: Vec<TypeArg> },
    Reference { mutable: bool, inner: Box<Type> },
    Array { inner: Box<Type>, len: Box<Expr> },
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Function { params: Vec<Type>, ret: Box<Type> },
    Dyn(Path),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeArg {
    Type(Type),
    Assoc { name: Ident, ty: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Path(Path),
    Call {
        callee: Box<Expr>,
        type_args: Vec<TypeArg>,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: String,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Unary {
        op: String,
        expr: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Block(Block),
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_branch: Option<Box<Expr>>,
    },
    StructLiteral {
        path: Path,
        fields: Vec<(String, Option<Expr>)>,
    },
    VecLiteral(Vec<Expr>),
    VecRepeat {
        value: Box<Expr>,
        count: Box<Expr>,
    },
    ErrorProp(Box<Expr>),
    Await(Box<Expr>),
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    Float(String),
    String(String),
    Char(String),
    Bool(bool),
}

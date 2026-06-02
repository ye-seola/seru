use logos::Span;

#[derive(Debug, Clone)]
pub struct ParsedStringPart {
    pub kind: ParsedStringPartKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ParsedStringPartKind {
    Text(String),
    Interpolation(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    String(String),
    Number(f32),
    Bool(bool),
    Null,
    Array(Vec<Expr>),
    Dict(Vec<(String, Expr)>),
    Ident(String),
    Bin {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    StringInterpolation(Vec<ParsedStringPart>),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Eq,
    Neq,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub struct SeruProgram {
    pub top_declarations: Vec<TopDeclaration>,
}

#[derive(Debug)]
pub enum TopDeclaration {
    ConstDeclaration(ConstDeclaration),
    ComponentDeclaration(ComponentDeclaration),
}

#[derive(Debug, Clone)]
pub struct ArgDeclaration {
    pub name: String,
    pub default: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct ConstDeclaration {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ComponentDeclaration {
    pub name: String,
    pub arg_declarations: Vec<ArgDeclaration>,
    pub body: ChildNode,
}

#[derive(Debug, Clone)]
pub enum ChildNode {
    ComponentCall(ComponentCall),
    For(ForBlock),
    If(IfBlock),
    Slot,
}

#[derive(Debug, Clone)]
pub struct ComponentCall {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Vec<ChildNode>,
}

#[derive(Debug, Clone)]
pub struct ForBlock {
    pub index_name: Option<String>,
    pub item_name: String,
    pub iterable: Expr,
    pub body: Vec<ChildNode>,
}

#[derive(Debug, Clone)]
pub struct IfBlock {
    pub condition: Expr,
    pub body: Vec<ChildNode>,
}

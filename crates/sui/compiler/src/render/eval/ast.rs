//! AST node definitions shared between the parser and the executor.

#[derive(Debug, Clone)]
pub enum Node {
    Null,
    Undefined,
    Bool(bool),
    Num(f64),
    Str(String),
    Template(Vec<TplPart>),
    Ident(String),
    Array(Vec<Node>),
    Object(Vec<(String, Node)>),
    Member { obj: Box<Node>, prop: String, optional: bool },
    Index { obj: Box<Node>, idx: Box<Node>, optional: bool },
    Call { callee: Box<Node>, args: Vec<Node> },
    Unary { op: UnaryOp, expr: Box<Node> },
    Binary { op: BinOp, l: Box<Node>, r: Box<Node> },
    Logical { op: LogicalOp, l: Box<Node>, r: Box<Node> },
    Conditional { cond: Box<Node>, then: Box<Node>, els: Box<Node> },
}

#[derive(Debug, Clone)]
pub enum TplPart {
    Str(String),
    Expr(Box<Node>),
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOp { Not, Neg, Pos }

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Lt, Le, Gt, Ge,
    Eq, Neq, EqStrict, NeqStrict,
}

#[derive(Debug, Copy, Clone)]
pub enum LogicalOp { And, Or, Coalesce }

// Myula compiler AST definitions
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-10: Initial version
//      26-02-11: Added more AST node types

#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ExprStatement(Box<Expression>),
    Declaration {
        names: Vec<String>,
        values: Vec<Expression>,
    },
    IfStmt {
        condition: Box<Expression>,
        then_branch: Vec<Statement>,
        elif_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    WhileStmt {
        condition: Box<Expression>,
        body: Vec<Statement>,
    },
    RepeatStmt {
        body: Vec<Statement>,
        condition: Box<Expression>,
    },
    ReturnStmt {
        values: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Literal(Literal),
    BinOp {
        left: Box<Expression>,
        operator: BinOp,
        right: Box<Expression>,
    },
    UnOp {
        operator: UnOp,
        operand: Box<Expression>,
    },
    FnCall {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    IndexOf {
        collection: Box<Expression>,
        index: Box<Expression>,
    },
    TableCtor {
        // {key: value, ...} - table
        // {value, value, ...} - arraylike, with implicit keys 1, 2, 3, ...
        // {key: value, value, ...} - mixed
        fields: Vec<(Option<Expression>, Expression)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
        name: Option<String>,
    },
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Concat,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
}

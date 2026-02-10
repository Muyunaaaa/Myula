// Myula compiler AST definitions
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
// 
// Changelog:
//      26-02-10: Initial version

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
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
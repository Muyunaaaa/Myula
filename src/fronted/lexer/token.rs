// Myula compiler token definitions
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-10: Initial version

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Errno,

    Eof,

    Ident(String),
    NumLit(f64),
    StrLit(String),

    Assign,

    Plus,
    Minus,
    Asterisk,
    Slash,
    Hat,
    Concat,

    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Comma,
    Dot,
    Semicolon,
    Colon,

    KwAnd,
    KwOr,
    KwNot,

    KwNil,
    KwTrue,
    KwFalse,

    KwIf,
    KwThen,
    KwElse,
    KwElseIf,
    KwEnd,
    KwWhile,
    KwDo,
    KwRepeat,
    KwUntil,
    KwFunction,
    KwReturn,
    KwLocal,
}
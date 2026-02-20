// Myula compiler token definitions
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-10: Initial version
//      26-02-13: Added '@' operator for legacy table ctor
//      26-02-20: Added '%' and '#' operators for modulo and length

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
    Percent,
    Hat,
    Hash,
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
    At,

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

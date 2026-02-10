// Myula compiler lexical analyzer
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-10: Initial version

pub mod token;

use std::vec::Vec;

use crate::fronted::lexer::token::Token;

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
    UnterminatedString,
    InvalidNumber,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    errors: Vec<LexerError>,
}

impl Lexer<'_> {
    pub fn new(input: &'_ str) -> Lexer<'_> {
        return Lexer {
            input: input,
            pos: 0,
            errors: vec![],
        };
    }

    pub fn get_err(&self) -> &Vec<LexerError> {
        return &self.errors;
    }

    pub fn get_pos(&self) -> usize {
        return self.pos;
    }

    fn emit_err(&mut self, err: LexerError) {
        self.errors.push(err);
    }

    fn is_eof(&self) -> bool {
        return self.pos >= self.input.len();
    }

    fn skip_ws(&mut self) {
        while !self.is_eof() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            self.skip_ws();
            if self.is_eof() {
                break;
            }
            if self.input.as_bytes()[self.pos] as char == '-' {
                if self.pos + 1 < self.input.len()
                    && self.input.as_bytes()[self.pos + 1] as char == '-'
                {
                    // single line comment
                    self.pos += 2;
                    while !self.is_eof() {
                        let c = self.input.as_bytes()[self.pos] as char;
                        if c == '\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.is_eof() {
            None
        } else {
            Some(self.input.as_bytes()[self.pos] as char)
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_eof() {
            None
        } else {
            let c = self.input.as_bytes()[self.pos] as char;
            self.pos += 1;
            Some(c)
        }
    }

    fn num_literal(&mut self) -> Token {
        let begin_pos = self.pos;
        loop {
            let c = self.peek_char();
            match c {
                Some(ch) if ch.is_ascii_digit() => {
                    self.advance();
                }
                _ => break,
            }
        };

        // fractional
        if self.peek_char() == Some('.') {
            self.advance(); // consume '.'
            loop {
                let c = self.peek_char();
                match c {
                    Some(ch) if ch.is_ascii_digit() => {
                        self.advance();
                    }
                    _ => break,
                }
            };
        }

        let num_str = &self.input[begin_pos..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Token::NumLit(num),
            Err(_) => {
                self.emit_err(LexerError::InvalidNumber);
                Token::NumLit(0.0)
            }
        }
    }

    fn str_literal(&mut self) -> Token {
        let quote_char = self.advance().unwrap(); // consume opening quote
        let mut escape = false;
        let begin_pos = self.pos;
        while !self.is_eof() {
            let c = self.advance().unwrap();
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == quote_char {
                // closing quote
                let str_lit = &self.input[begin_pos..self.pos - 1];
                return Token::StrLit(str_lit.to_string());
            }
        }
        self.emit_err(LexerError::UnterminatedString);
        Token::StrLit(String::new())
    }

    fn is_keyword(s: &str) -> Option<Token> {
        match s {
            "and" => Some(Token::KwAnd),
            "or" => Some(Token::KwOr),
            "not" => Some(Token::KwNot),
            "nil" => Some(Token::KwNil),
            "true" => Some(Token::KwTrue),
            "false" => Some(Token::KwFalse),
            "if" => Some(Token::KwIf),
            "then" => Some(Token::KwThen),
            "else" => Some(Token::KwElse),
            "elseif" => Some(Token::KwElseIf),
            "end" => Some(Token::KwEnd),
            "while" => Some(Token::KwWhile),
            "do" => Some(Token::KwDo),
            "repeat" => Some(Token::KwRepeat),
            "until" => Some(Token::KwUntil),
            "function" => Some(Token::KwFunction),
            "return" => Some(Token::KwReturn),
            "local" => Some(Token::KwLocal),
            _ => None,
        }
    }

    fn ident_or_keyword(&mut self) -> Token {
        let begin_pos = self.pos;
        loop {
            let c = self.peek_char();
            match c {
                Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' => {
                    self.advance();
                }
                _ => break,
            }
        };
        let ident_str = &self.input[begin_pos..self.pos];
        if let Some(kw_token) = Lexer::is_keyword(ident_str) {
            kw_token
        } else {
            Token::Ident(ident_str.to_string())
        }
    }

    fn double_char_op(&mut self, second: char, double_token: Token, single_token: Token) -> Token {
        // already consumed first char
        if self.peek_char() == Some(second) {
            self.advance(); // consume second char
            double_token
        } else {
            single_token
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws_and_comments();

        if self.is_eof() {
            return Token::Eof;
        }

        let c = self.peek_char();
        match c {
            Some(ch) if ch.is_ascii_digit() => self.num_literal(),
            Some('"') | Some('\'') => self.str_literal(),
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => self.ident_or_keyword(),
            _ => {
                match self.advance() {
                    Some(chr) => {
                        match chr {
                            '+' => Token::Plus,
                            '-' => Token::Minus,
                            '*' => Token::Asterisk,
                            '/' => Token::Slash,
                            '^' => Token::Hat,
                            '.' => self.double_char_op('.', Token::Concat, Token::Dot),
                            '=' => self.double_char_op('=', Token::Eq, Token::Assign),
                            '~' => self.double_char_op('=', Token::Neq, Token::Errno),
                            '<' => self.double_char_op('=', Token::Leq, Token::Lt),
                            '>' => self.double_char_op('=', Token::Geq, Token::Gt),
                            '(' => Token::LParen,
                            ')' => Token::RParen,
                            '{' => Token::LBrace,
                            '}' => Token::RBrace,
                            '[' => Token::LBracket,
                            ']' => Token::RBracket,
                            ',' => Token::Comma,
                            ';' => Token::Semicolon,
                            ':' => Token::Colon,
                            other => {
                                self.emit_err(LexerError::UnexpectedCharacter(other));
                                Token::Errno
                            }
                        }
                    }
                    None => {
                        // should not reach here
                        unreachable!()
                    }
                }
            }
        }
    }
}
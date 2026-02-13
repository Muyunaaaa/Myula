// Myula compiler parser
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-10: Initial version
//      26-02-11: Minor fixes
//      26-02-11: Added function call and indexing parsing
//      26-02-11: Added function declaration parsing

pub mod ast;

use crate::frontend::lexer::{Lexer, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorType {
    UnexpectedToken,
    UnclosedBrackets,
    UnexpectedEof,
    InvalidExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    pub err_type: ParserErrorType,
    pub message: String,
    pub pos: usize,
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    current_token: Option<Token>,
    next_token: Option<Token>,
    errors: Vec<ParserError>,
}

impl Parser<'_> {
    pub fn new<'a>(lexer: &'a mut Lexer<'a>) -> Parser<'a> {
        let next = lexer.next_token();
        return Parser {
            lexer: lexer,
            current_token: None,
            next_token: Some(next),
            errors: vec![],
        };
    }

    pub fn get_err(&self) -> &Vec<ParserError> {
        return &self.errors;
    }

    pub fn get_lexer<'a>(&self) -> &Lexer<'_> {
        return &self.lexer;
    }

    fn emit_err(&mut self, err_type: ParserErrorType, message: String) {
        let pos = self.lexer.get_pos();
        self.errors.push(ParserError {
            err_type: err_type,
            message: message,
            pos: pos,
        });
    }

    fn advance_tokens(&mut self) {
        self.current_token = self.next_token.take();
        self.next_token = Some(self.lexer.next_token());
    }

    fn peek_token(&self) -> &Token {
        if let Some(tok) = &self.next_token {
            return tok;
        } else {
            return &Token::Eof;
        }
    }

    #[allow(dead_code)]
    fn current_token(&self) -> &Token {
        if let Some(tok) = &self.current_token {
            return tok;
        } else {
            return &Token::Eof;
        }
    }

    fn expect(&mut self, expected: Token) -> bool {
        if self.peek_token() == &expected {
            self.advance_tokens();
            return true;
        } else {
            let msg = format!(
                "Expected token {:?}, but found {:?}",
                expected,
                self.peek_token()
            );
            self.emit_err(ParserErrorType::UnexpectedToken, msg);
            return false;
        }
    }

    fn binop_precedence(op: &ast::BinOp) -> Option<u8> {
        match op {
            ast::BinOp::Assign => Some(0),
            ast::BinOp::Or => Some(1),
            ast::BinOp::And => Some(2),
            ast::BinOp::Eq
            | ast::BinOp::Neq
            | ast::BinOp::Lt
            | ast::BinOp::Gt
            | ast::BinOp::Leq
            | ast::BinOp::Geq => Some(3),
            ast::BinOp::Add | ast::BinOp::Sub => Some(4),
            ast::BinOp::Mul | ast::BinOp::Div => Some(5),
            ast::BinOp::Pow => Some(6),
            _ => None,
        }
    }

    fn is_binop_right_assoc(op: &ast::BinOp) -> bool {
        match op {
            ast::BinOp::Pow | ast::BinOp::Concat => true,
            _ => false,
        }
    }

    fn token_to_ast_binop(token: &Token) -> Option<ast::BinOp> {
        match token {
            Token::Plus => Some(ast::BinOp::Add),
            Token::Minus => Some(ast::BinOp::Sub),
            Token::Asterisk => Some(ast::BinOp::Mul),
            Token::Slash => Some(ast::BinOp::Div),
            Token::Hat => Some(ast::BinOp::Pow),
            Token::Concat => Some(ast::BinOp::Concat),
            Token::Eq => Some(ast::BinOp::Eq),
            Token::Neq => Some(ast::BinOp::Neq),
            Token::Lt => Some(ast::BinOp::Lt),
            Token::Gt => Some(ast::BinOp::Gt),
            Token::Leq => Some(ast::BinOp::Leq),
            Token::Geq => Some(ast::BinOp::Geq),
            Token::KwAnd => Some(ast::BinOp::And),
            Token::KwOr => Some(ast::BinOp::Or),
            Token::Assign => Some(ast::BinOp::Assign),
            _ => None,
        }
    }

    fn parse_fn_call_expression(&mut self, callee: ast::Expression) -> Option<ast::Expression> {
        self.advance_tokens(); // consume '('

        // args
        let mut args: Vec<ast::Expression> = vec![];
        if self.peek_token() != &Token::RParen {
            loop {
                let arg_expr = self.parse_expression();
                if arg_expr.is_none() {
                    self.emit_err(
                        ParserErrorType::InvalidExpression,
                        "Function call argument requires a valid expression".to_string(),
                    );
                    return None;
                }
                args.push(arg_expr.unwrap());

                if self.peek_token() == &Token::Comma {
                    self.advance_tokens(); // consume ','
                    continue;
                } else {
                    break;
                }
            }
        }

        if !self.expect(Token::RParen) {
            self.emit_err(
                ParserErrorType::UnclosedBrackets,
                "Expected ')' after function call arguments".to_string(),
            );
            return None;
        }

        Some(ast::Expression::FnCall {
            callee: Box::new(callee),
            arguments: args,
        })
    }

    fn parse_index_expression(&mut self, collection: ast::Expression) -> Option<ast::Expression> {
        self.advance_tokens(); // consume '['
        let index_expr = self.parse_expression();
        if index_expr.is_none() {
            self.emit_err(
                ParserErrorType::InvalidExpression,
                "Index expression requires a valid expression".to_string(),
            );
            return None;
        }
        if !self.expect(Token::RBracket) {
            self.emit_err(
                ParserErrorType::UnclosedBrackets,
                "Expected ']' after index expression".to_string(),
            );
            return None;
        }

        Some(ast::Expression::IndexOf {
            collection: Box::new(collection),
            index: Box::new(index_expr.unwrap()),
        })
    }

    fn parse_table_ctor(&mut self, is_legacy: bool) -> Option<ast::Expression> {
        // Lua 1.1 legacy table constructor with '@' operator
        if is_legacy {
            self.expect(Token::At);
        }

        self.expect(Token::LBrace);
        let mut fields: Vec<(Option<ast::Expression>, ast::Expression)> = vec![];
        if self.peek_token() != &Token::RBrace {
            loop {
                let key_expr: Option<ast::Expression>;
                let value_expr: ast::Expression;

                // check if it's a key-value pair or just a value
                if self.peek_token() == &Token::LBracket {
                    // modern lua
                    // key-value pair with expression key
                    // {[ 1 + 1 ] = value, ...}
                    self.advance_tokens(); // consume '['
                    let key = self.parse_expression();
                    if key.is_none() {
                        self.emit_err(
                            ParserErrorType::InvalidExpression,
                            "Table constructor key requires a valid expression".to_string(),
                        );
                        return None;
                    }
                    key_expr = Some(key.unwrap());
                    if !self.expect(Token::RBracket) {
                        self.emit_err(
                            ParserErrorType::UnclosedBrackets,
                            "Expected ']' after table constructor key".to_string(),
                        );
                        return None;
                    }
                    if !self.expect(Token::Assign) {
                        self.emit_err(
                            ParserErrorType::UnexpectedToken,
                            "Expected '=' after table constructor key".to_string(),
                        );
                        return None;
                    }
                } else if let Token::Ident(_) = self.peek_token() {
                    // key-value pair with identifier key
                    // { key = value, ... }
                    let key = match self.peek_token().clone() {
                        Token::Ident(name) => {
                            self.advance_tokens();
                            name
                        }
                        _ => unreachable!(),
                    };

                    // for this style of key, we convert it to string literal
                    key_expr = Some(ast::Expression::Literal(ast::Literal::String(key)));

                    if !self.expect(Token::Assign) {
                        self.emit_err(
                            ParserErrorType::UnexpectedToken,
                            "Expected '=' after table constructor key".to_string(),
                        );
                        return None;
                    }
                } else {
                    // just a value
                    // arraylike
                    // { value, value, ... }
                    key_expr = None;
                }

                let value = self.parse_expression();
                if value.is_none() {
                    self.emit_err(
                        ParserErrorType::InvalidExpression,
                        "Table constructor value requires a valid expression".to_string(),
                    );
                    return None;
                }
                value_expr = value.unwrap();

                fields.push((key_expr, value_expr));

                if self.peek_token() == &Token::Comma {
                    self.advance_tokens(); // consume ','
                    continue;
                } else {
                    break;
                }
            }
        }

        self.expect(Token::RBrace);
        Some(ast::Expression::TableCtor { fields })
    }

    fn parse_unary_or_primary_expression(&mut self) -> Option<ast::Expression> {
        let token = self.peek_token().clone();
        let simple = match token {
            // unary operators
            Token::Minus => {
                self.advance_tokens();
                let operand = self.parse_unary_or_primary_expression()?;
                Some(ast::Expression::UnOp {
                    operator: ast::UnOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Token::Plus => {
                self.advance_tokens();
                let operand = self.parse_unary_or_primary_expression()?;
                Some(ast::Expression::UnOp {
                    operator: ast::UnOp::Pos,
                    operand: Box::new(operand),
                })
            }
            Token::KwNot => {
                self.advance_tokens();
                let operand = self.parse_unary_or_primary_expression()?;
                Some(ast::Expression::UnOp {
                    operator: ast::UnOp::Not,
                    operand: Box::new(operand),
                })
            }

            // other
            Token::Ident(name) => {
                self.advance_tokens();
                Some(ast::Expression::Identifier(name))
            }
            Token::NumLit(num) => {
                self.advance_tokens();
                Some(ast::Expression::Literal(ast::Literal::Number(num)))
            }
            Token::StrLit(s) => {
                self.advance_tokens();
                Some(ast::Expression::Literal(ast::Literal::String(s)))
            }
            Token::KwTrue => {
                self.advance_tokens();
                Some(ast::Expression::Literal(ast::Literal::Boolean(true)))
            }
            Token::KwFalse => {
                self.advance_tokens();
                Some(ast::Expression::Literal(ast::Literal::Boolean(false)))
            }
            Token::KwNil => {
                self.advance_tokens();
                Some(ast::Expression::Literal(ast::Literal::Nil))
            }

            // parentheses
            Token::LParen => {
                self.advance_tokens(); // consume '('
                let expr = self.parse_expression()?;
                if !self.expect(Token::RParen) {
                    return None;
                }
                Some(expr)
            }

            // function literal
            Token::KwFunction => {
                // function literal
                let func_lit = self.parse_function_decl_expression()?;
                Some(func_lit)
            }

            // table constructor
            // '{' or '@' for legacy
            Token::LBrace | Token::At => {
                let is_legacy = token == Token::At;
                let table_ctor = self.parse_table_ctor(is_legacy)?;
                Some(table_ctor)
            }

            _ => {
                let msg = format!("Unexpected token {:?} in expression", token);
                self.emit_err(ParserErrorType::InvalidExpression, msg);
                None
            } // todo: fn calls, table ctors
        };

        if simple.is_none() {
            return None;
        }
        let mut simple = simple.unwrap();

        // postfix exprs: fn calls, indexing
        loop {
            let next_tok = self.peek_token().clone();
            match next_tok {
                Token::LParen => {
                    // fn call
                    let fn_call_expr = self.parse_fn_call_expression(simple);
                    if fn_call_expr.is_none() {
                        return None;
                    }

                    if let Some(expr) = fn_call_expr {
                        simple = expr;
                    } else {
                        return None;
                    }
                }
                Token::LBracket => {
                    // indexing
                    let index_expr = self.parse_index_expression(simple);
                    if index_expr.is_none() {
                        return None;
                    }
                    if let Some(expr) = index_expr {
                        simple = expr;
                    } else {
                        return None;
                    }
                }
                _ => break,
            }
        }

        Some(simple)
    }

    fn parse_binary_expression_impl(&mut self, min_prec: u8) -> Option<ast::Expression> {
        let lhs = self.parse_unary_or_primary_expression();
        if lhs.is_none() {
            self.emit_err(
                ParserErrorType::InvalidExpression,
                "Failed to parse left-hand side expression".to_string(),
            );
            return None;
        }

        let mut left_expr = lhs.unwrap();

        loop {
            let op = Parser::token_to_ast_binop(self.peek_token());
            if op.is_none() {
                break;
            }
            let op = op.unwrap();

            let prec = Parser::binop_precedence(&op);
            if prec.is_none() {
                break;
            }
            let prec = prec.unwrap();

            if prec < min_prec {
                break;
            }

            self.advance_tokens(); // consume operator

            let mut next_min_prec = prec;
            if !Parser::is_binop_right_assoc(&op) {
                next_min_prec += 1;
            }

            let rhs = self.parse_binary_expression_impl(next_min_prec);
            if rhs.is_none() {
                break;
            }
            let rhs = rhs.unwrap();

            left_expr = ast::Expression::BinOp {
                left: Box::new(left_expr),
                operator: op,
                right: Box::new(rhs),
            };
        }

        return Some(left_expr);
    }

    fn parse_binary_expression(&mut self) -> Option<ast::Expression> {
        self.parse_binary_expression_impl(0)
    }

    fn parse_expression(&mut self) -> Option<ast::Expression> {
        self.parse_binary_expression()
    }

    fn parse_function_decl_inner(&mut self) -> Option<(Vec<String>, Vec<ast::Statement>)> {
        self.expect(Token::LParen);

        // parameters
        let mut params: Vec<String> = vec![];
        if self.peek_token() != &Token::RParen {
            loop {
                match self.peek_token().clone() {
                    Token::Ident(param_name) => {
                        params.push(param_name);
                        self.advance_tokens();
                        if self.peek_token() == &Token::Comma {
                            self.advance_tokens(); // consume ','
                            continue;
                        } else {
                            break;
                        }
                    }
                    _ => {
                        let msg = format!(
                            "Expected identifier in function parameters, found {:?}",
                            self.peek_token()
                        );
                        self.emit_err(ParserErrorType::UnexpectedToken, msg);
                        return None;
                    }
                }
            }
        }

        self.expect(Token::RParen);

        // function body
        let mut body: Vec<ast::Statement> = vec![];
        while self.peek_token() != &Token::KwEnd {
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.expect(Token::KwEnd);

        Some((params, body))
    }

    fn parse_function_decl_statement(&mut self, is_local: bool) -> Option<ast::Statement> {
        // dont expect local here, handled in local decl

        self.expect(Token::KwFunction);

        // function name
        let name = match self.peek_token().clone() {
            Token::Ident(func_name) => {
                self.advance_tokens();
                func_name
            }
            _ => {
                let msg = format!(
                    "Expected function name identifier, found {:?}. Note that anonymous functions not \
                    bound to any variable are meaningless!",
                    self.peek_token()
                );
                self.emit_err(ParserErrorType::UnexpectedToken, msg);
                return None;
            }
        };

        let (params, body) = self.parse_function_decl_inner()?;

        // for named functions, we treat them as assignment to a function literal
        let func_literal = ast::Expression::Literal(ast::Literal::Function {
            name: Some(name.clone()),
            params,
            body,
        });

        if is_local {
            // local decl
            Some(ast::Statement::Declaration {
                names: vec![name],
                values: vec![func_literal],
            })
        } else {
            // assignment
            // global decl actually
            Some(ast::Statement::ExprStatement(Box::new(
                ast::Expression::BinOp {
                    left: Box::new(ast::Expression::Identifier(name)),
                    operator: ast::BinOp::Assign,
                    right: Box::new(func_literal),
                },
            )))
        }
    }

    fn parse_function_decl_expression(&mut self) -> Option<ast::Expression> {
        self.expect(Token::KwFunction);

        let (params, body) = self.parse_function_decl_inner()?;

        Some(ast::Expression::Literal(ast::Literal::Function {
            params,
            body,
            name: None,
        }))
    }

    fn parse_local_decl_statement(&mut self) -> Option<ast::Statement> {
        self.expect(Token::KwLocal);

        if self.peek_token() == &Token::KwFunction {
            // local function declaration
            return self.parse_function_decl_statement(true);
        }

        let mut names: Vec<String> = vec![];
        loop {
            match self.peek_token().clone() {
                Token::Ident(name) => {
                    names.push(name);
                    self.advance_tokens();
                    if self.peek_token() == &Token::Comma {
                        self.advance_tokens(); // consume ','
                        continue;
                    } else {
                        break;
                    }
                }
                _ => {
                    let msg = format!(
                        "Expected identifier in local declaration, found {:?}",
                        self.peek_token()
                    );
                    self.emit_err(ParserErrorType::UnexpectedToken, msg);
                    return None;
                }
            }
        }

        self.expect(Token::Assign);

        let mut values: Vec<ast::Expression> = vec![];
        loop {
            let expr = self.parse_expression()?;
            values.push(expr);
            if self.peek_token() == &Token::Comma {
                self.advance_tokens(); // consume ','
                continue;
            } else {
                break;
            }
        }

        return Some(ast::Statement::Declaration { names, values });
    }

    fn parse_if_statement(&mut self) -> Option<ast::Statement> {
        self.expect(Token::KwIf);
        let cond = self.parse_expression()?;

        self.expect(Token::KwThen);
        let mut then_branch: Vec<ast::Statement> = vec![];
        while self.peek_token() != &Token::KwElse
            && self.peek_token() != &Token::KwElseIf
            && self.peek_token() != &Token::KwEnd
        {
            if let Some(stmt) = self.parse_statement() {
                then_branch.push(stmt);
            } else {
                break;
            }
        }

        let mut elif_branches: Vec<(ast::Expression, Vec<ast::Statement>)> = vec![];
        while self.peek_token() == &Token::KwElseIf {
            self.advance_tokens(); // consume 'elseif'
            let elif_cond = self.parse_expression()?;
            self.expect(Token::KwThen);
            let mut elif_branch: Vec<ast::Statement> = vec![];
            while self.peek_token() != &Token::KwElse
                && self.peek_token() != &Token::KwElseIf
                && self.peek_token() != &Token::KwEnd
            {
                if let Some(stmt) = self.parse_statement() {
                    elif_branch.push(stmt);
                } else {
                    break;
                }
            }
            elif_branches.push((elif_cond, elif_branch));
        }

        let else_branch = if self.peek_token() == &Token::KwElse {
            self.advance_tokens(); // consume 'else'
            let mut else_branch: Vec<ast::Statement> = vec![];
            while self.peek_token() != &Token::KwEnd {
                if let Some(stmt) = self.parse_statement() {
                    else_branch.push(stmt);
                } else {
                    break;
                }
            }
            Some(else_branch)
        } else {
            None
        };
        self.expect(Token::KwEnd);

        Some(ast::Statement::IfStmt {
            condition: Box::new(cond),
            then_branch,
            elif_branches,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Option<ast::Statement> {
        self.expect(Token::KwWhile);
        let condition = self.parse_expression()?;
        self.expect(Token::KwDo);

        let mut body: Vec<ast::Statement> = vec![];
        while self.peek_token() != &Token::KwEnd {
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.expect(Token::KwEnd);
        Some(ast::Statement::WhileStmt {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_repeat_statement(&mut self) -> Option<ast::Statement> {
        self.expect(Token::KwRepeat);

        let mut body: Vec<ast::Statement> = vec![];
        while self.peek_token() != &Token::KwUntil {
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.expect(Token::KwUntil);
        let condition = self.parse_expression()?;
        Some(ast::Statement::RepeatStmt {
            body,
            condition: Box::new(condition),
        })
    }

    fn parse_return_statement(&mut self) -> Option<ast::Statement> {
        self.expect(Token::KwReturn);

        let mut values: Vec<ast::Expression> = vec![];
        loop {
            let expr = self.parse_expression();
            if expr.is_none() {
                // no more expressions
                break;
            }
            let expr = expr.unwrap();

            values.push(expr);
            if self.peek_token() == &Token::Comma {
                self.advance_tokens(); // consume ','
                continue;
            } else {
                break;
            }
        }

        Some(ast::Statement::ReturnStmt { values })
    }

    fn parse_statement(&mut self) -> Option<ast::Statement> {
        let next_tok = self.peek_token().clone();
        match next_tok {
            Token::KwLocal => self.parse_local_decl_statement(),
            Token::KwIf => self.parse_if_statement(),
            Token::KwWhile => self.parse_while_statement(),
            Token::KwRepeat => self.parse_repeat_statement(),
            Token::KwFunction => {
                // for local function declarations, handled in local decl
                self.parse_function_decl_statement(false)
            }
            Token::KwReturn => self.parse_return_statement(),
            _ => {
                // default is expression statement
                self.parse_expression()
                    .map(|expr| ast::Statement::ExprStatement(Box::new(expr)))
            }
        }
    }

    fn parse_program(&mut self) -> ast::Program {
        let mut body: Vec<ast::Statement> = vec![];
        loop {
            if self.peek_token() == &Token::Eof {
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }
        return ast::Program { body: body };
    }

    pub fn parse(&mut self) -> ast::Program {
        return self.parse_program();
    }
}

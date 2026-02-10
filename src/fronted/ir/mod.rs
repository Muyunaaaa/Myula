use std::collections::{HashSet, HashMap};

use crate::fronted::parser;

pub struct IRGenerator {
    ir: Vec<IRInstruction>,
    errors: Vec<IRGeneratorError>,
    next_reg: usize,

    global_scope: IRGlobalScope,
    scope_stack: Vec<IRScope>,
}

#[derive(Debug, Clone)]
pub struct IRScope {
    variables: HashMap<String, usize>, // variable name -> register number
}

#[derive(Debug, Clone)]
pub struct IRGlobalScope {
    variables: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum IRGeneratorError {
    UndefinedVariable(String),
}

#[derive(Debug, Clone)]
pub enum IROperand {
    Reg(usize),
    ImmFloat(f64),
    ImmBool(bool),
    ImmStr(String),
    Nil,
    Unit,
}

impl IROperand {
    pub fn to_string(&self) -> String {
        match self {
            IROperand::Reg(reg) => format!("%{}", reg),
            IROperand::ImmFloat(f) => format!("${}", f),
            IROperand::ImmBool(b) => format!("${}", b),
            IROperand::ImmStr(s) => format!("$\"{}\"", s),
            IROperand::Nil => "$nil".to_string(),
            IROperand::Unit => "$unit".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRBinOp {
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
}

impl IRBinOp {
    pub fn to_string(&self) -> String {
        return format!("{:?}", self).to_lowercase();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRUnOp {
    Neg,
    Not,
}

impl IRUnOp {
    pub fn to_string(&self) -> String {
        return format!("{:?}", self).to_lowercase();
    }
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    // %dest = op %src1, %src2
    Binary {
        dest: usize,
        src1: IROperand,
        src2: IROperand,
        operator: IRBinOp,
    },
    // %dest = op %src
    Unary {
        dest: usize,
        operator: IRUnOp,
        src: IROperand,
    },
    // %dest = LoadGlobal "name"
    LoadGlobal {
        dest: usize,
        name: String,
    },
    // StoreGlobal "name" %src
    StoreGlobal {
        name: String,
        src: IROperand,
    },
    // %dest = Call %callee, [args]
    Call {
        dest: usize,
        callee: IROperand,
        args: Vec<IROperand>,
    },
}

impl IRInstruction {
    pub fn to_string(&self) -> String {
        match self {
            IRInstruction::Binary { dest, src1, src2, operator } => {
                format!("%{} = {} {} {}",
                        dest,
                        operator.to_string(),
                        src1.to_string(),
                        src2.to_string())
            }
            IRInstruction::Unary { dest, operator, src } => {
                format!("%{} = {} {}",
                        dest,
                        operator.to_string(),
                        src.to_string())
            }
            IRInstruction::LoadGlobal { dest, name } => {
                format!("%{} = LoadGlobal \"{}\"", dest, name)
            }
            IRInstruction::StoreGlobal { name, src } => {
                format!("StoreGlobal \"{}\" {}", name, src.to_string())
            }
            IRInstruction::Call { dest, callee, args } => {
                let args_str =
                    args
                        .iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                format!("%{} = Call {}, [{}]", dest, callee.to_string(), args_str)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRTerminator {
    Return(Vec<IROperand>),
    // br %label
    Jump(usize),
    // br %cond, %then_label, %else_label
    Branch {
        cond: IROperand,
        br_true: usize,
        br_false: usize,
    }
}

#[derive(Debug, Clone)]
pub struct IRBasicBlock {
    pub id: usize,
    pub instructions: Vec<IRInstruction>,
    pub terminator: IRTerminator,
}

impl IRGenerator {
    pub fn new() -> IRGenerator {
        return IRGenerator {
            ir: vec![],
            errors: vec![],
            next_reg: 0,
            global_scope: IRGlobalScope {
                variables: HashSet::new(),
            },
            scope_stack: vec![],
        };
    }

    pub fn get_ir(&self) -> &Vec<IRInstruction> {
        return &self.ir;
    }

    pub fn get_err(&self) -> &Vec<IRGeneratorError> {
        return &self.errors;
    }

    fn emit(&mut self, instr: IRInstruction) {
        self.ir.push(instr);
    }

    fn emit_err(&mut self, err: IRGeneratorError) {
        self.errors.push(err);
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(IRScope {
            variables: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn decl_global(&mut self, name: String) {
        self.global_scope.variables.insert(name);
    }

    fn decl_local(&mut self, name: String) {
        let reg = self.alloc_reg();
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.variables.insert(name, reg);
        } else {
            panic!("No scope to declare local variable");
        }
    }

    fn is_var_declared(&self, name: &str) -> bool {
        for scope in self.scope_stack.iter().rev() {
            if scope.variables.contains_key(name) {
                return true;
            }
        }
        return self.global_scope.variables.contains(name);
    }

    fn alloc_reg(&mut self) -> usize {
        let reg = self.next_reg;
        self.next_reg += 1;
        return reg;
    }

    fn generate_expr(&mut self, expr: &parser::ast::Expression) -> IROperand {
        match expr {
            parser::ast::Expression::Identifier(name) => {
                if !self.is_var_declared(name) {
                    self.emit_err(IRGeneratorError::UndefinedVariable(name.clone()));
                }

                let reg = self.alloc_reg();
                self.emit(IRInstruction::LoadGlobal {
                    dest: reg,
                    name: name.clone(),
                });
                return IROperand::Reg(reg);
            }
            parser::ast::Expression::Literal(lit) => {
                match lit {
                    parser::ast::Literal::Number(n) => return IROperand::ImmFloat(*n),
                    parser::ast::Literal::String(s) => return IROperand::ImmStr(s.clone()),
                    parser::ast::Literal::Boolean(b) => return IROperand::ImmBool(*b),
                    parser::ast::Literal::Nil => return IROperand::Nil,
                }
            }
            _ => unimplemented!(),
        }
    }

    fn generate_stmt(&mut self, stmt: &parser::ast::Statement) {
        match stmt {
            parser::ast::Statement::ExprStatement(expr) => {
                self.generate_expr(expr);
            }
            parser::ast::Statement::Declaration { names, values } => {
                for (name, value) in names.iter().zip(values.iter()) {
                    let src = self.generate_expr(value);
                    // by default, 'Declaration' is for local variables
                    self.decl_local(name.clone());
                    self.emit(IRInstruction::StoreGlobal {
                        name: name.clone(),
                        src,
                    });
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn generate(&mut self, program: &parser::ast::Program) {
        // 'global' local scope
        self.push_scope();
        for stmt in &program.body {
            self.generate_stmt(stmt);
        }
        self.pop_scope();
    }
}
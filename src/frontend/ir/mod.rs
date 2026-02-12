// Myula compiler SSA IR generator
// Created by: Zimeng Li <zimengli@mail.nwpu.edu.cn>
//
// Changelog:
//      26-02-11: Initial version
//      26-02-11: Added calling and indexing support
//      26-02-11: Added function declaration support, both named and anonymous
//      26-02-11: Some documentation
//      26-02-12: Added FallThrough terminator
//      26-02-12: [Breaking Change]
//                AddrLocal instruction renamed from AllocLocal
//                to better reflect its purpose
//                Provided details about local vars in function prototype
//     26-02-12: Added LoadImm instruction for loading immediate values
//     26-02-12: Added Drop instruction for discarding values that are not needed

use std::collections::{HashMap, HashSet};

use crate::frontend::parser;

pub struct IRGenerator {
    module: IRModule,
    function_contexts: Vec<IRFunctionContext>,

    next_anonymous_func_id: usize,

    errors: Vec<IRGeneratorError>,

    scope_stack: Vec<IRScope>,
}

#[derive(Debug, Clone)]
pub struct IRScope {
    local_variables: HashMap<String, IRLocalVarSlot>, // local variable name -> slot number
}

type IRLocalVarSlot = usize;

#[derive(Debug, Clone)]
struct IRFunctionContext {
    name: String,
    params: Vec<String>,

    active_block: Option<IRActiveBlock>,
    basic_blocks: Vec<IRBasicBlock>,

    next_reg: usize,
    next_block_id: usize,
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
    // virtual register
    Reg(usize),
    // Local variable slot, used for AddrLocal instruction,
    // DO NOT use this directly as an operand in any other instructions
    Slot(IRLocalVarSlot),
    // function prototype name
    // you can think of it as a function type
    // do not use it directly except in FnProto instruction
    // to use the underlying function, you need to instantiate it first
    // with FnProto instruction
    Proto(String),
    // for constants, when emitting bytecode,
    // the values should be put into constant pool
    // Immediate values should only be used in LoadImm instruction
    ImmFloat(f64),  // immediate float value
    ImmBool(bool),  // immediate boolean value
    ImmStr(String), // immediate string value
    Nil,            // nil value
    Unit,           // unit value
}

impl IROperand {
    pub fn to_string(&self) -> String {
        match self {
            IROperand::Reg(reg) => format!("%{}", reg),
            IROperand::Proto(name) => format!("@{}", name),
            IROperand::Slot(slot) => format!("%local_{}", slot),
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
    And,
    Or,
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
    // %dest = LoadImm $value
    // load an immediate value into %dest
    LoadImm {
        dest: usize,
        value: IROperand,
    },
    // %dest = op %src1, %src2
    // for binary operations
    // calculate the result of src1 op src2 and store in dest
    // where dest is a virtual register
    Binary {
        dest: usize,
        src1: IROperand,
        src2: IROperand,
        operator: IRBinOp,
    },
    // %dest = op %src
    // for unary operations
    // calculate the result of op src and store in dest
    // where dest is a virtual register
    Unary {
        dest: usize,
        operator: IRUnOp,
        src: IROperand,
    },
    // %dest = AddrLocal %local_slot
    // get the address of a local variable by its slot number, store in %dest
    // this is used when address of a local variable is needed,
    // such as in StoreLocal and LoadLocal instructions
    //
    // see: IRFunction::local_variables for mapping information
    //      of local variable names to slot numbers
    AddrLocal {
        dest: usize,
        slot: IROperand,
    },
    // %dest = LoadLocal (%src)
    // load the value of local variable at *address* %src into %dest
    // %src is the *ADDRESS* of the local variable, *not* the slot number,
    // so to use this, you need to AddrLocal first!!
    LoadLocal {
        dest: usize,
        src: IROperand,
    },
    // StoreLocal (%dst), %src
    // store the value of %src into local variable at *address* %dst
    // returns the value stored, which is useful for chained assignments like a = b = c
    //
    // %dst is the *ADDRESS* of the local variable, *NOT* the slot number,
    // so to use this, you need to AddrLocal first, like in LoadLocal
    StoreLocal {
        dest: usize,
        dst: usize,
        src: IROperand,
    },
    // %dest = LoadGlobal "name"
    // load the value of global variable "name" into %dest
    LoadGlobal {
        dest: usize,
        name: String,
    },
    // StoreGlobal "name" %src
    // store the value of %src into global variable "name"
    // returns the value stored, same as StoreLocal, for chained assignment support
    StoreGlobal {
        dest: usize,
        name: String,
        src: IROperand,
    },
    // %nil = Drop %src
    // drop the value in register %src, used for discarding values that are not needed
    // for example, the return value of a function call that is not used,
    // or the value of an expression statement
    // %nil is used as a dummy destination
    //
    // for reg-based bytecode, this can be interpreted as an nop,
    // but for stack-based bytecode, this can be used to pop the value from the stack
    Drop {
        src: IROperand,
    },
    // %dest = Call %callee, [args]
    // Invoke function %callee with arguments [args],
    // store the return value into %dest
    Call {
        dest: usize,
        callee: IROperand,
        args: Vec<IROperand>,
    },
    // %dest = IndexOf %collection, %index
    // Get the element at %index from %collection,
    IndexOf {
        dest: usize,
        collection: IROperand,
        index: IROperand,
    },
    // %dest = FnProto @func_name
    // Instantiate a function prototype @func_name,
    // store the function reference into %dest
    // todo: limitations in handling upvalues
    FnProto {
        dest: usize,
        func_proto: IROperand,
    },
}

impl IRInstruction {
    pub fn to_string(&self) -> String {
        match self {
            IRInstruction::LoadImm { dest, value } => {
                format!("%{} = LoadImm {}", dest, value.to_string())
            }
            IRInstruction::Binary {
                dest,
                src1,
                src2,
                operator,
            } => {
                format!(
                    "%{} = {} {} {}",
                    dest,
                    operator.to_string(),
                    src1.to_string(),
                    src2.to_string()
                )
            }
            IRInstruction::Unary {
                dest,
                operator,
                src,
            } => {
                format!("%{} = {} {}", dest, operator.to_string(), src.to_string())
            }
            IRInstruction::AddrLocal { dest, slot } => {
                format!("%{} = AddrLocal {}", dest, slot.to_string())
            }
            IRInstruction::LoadLocal { dest, src } => {
                format!("%{} = LoadLocal ({})", dest, src.to_string())
            }
            IRInstruction::StoreLocal { dest, dst, src } => {
                format!("%{} = StoreLocal (%{}) {}", dest, dst, src.to_string())
            }
            IRInstruction::LoadGlobal { dest, name } => {
                format!("%{} = LoadGlobal \"{}\"", dest, name)
            }
            IRInstruction::StoreGlobal { dest, name, src } => {
                format!("%{} = StoreGlobal \"{}\" {}", dest, name, src.to_string())
            }
            IRInstruction::Drop { src } => {
                format!("%nil = Drop {}", src.to_string())
            }
            IRInstruction::Call { dest, callee, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("%{} = Call {}, [{}]", dest, callee.to_string(), args_str)
            }
            IRInstruction::IndexOf {
                dest,
                collection,
                index,
            } => {
                format!(
                    "%{} = IndexOf {}, {}",
                    dest,
                    collection.to_string(),
                    index.to_string()
                )
            }
            IRInstruction::FnProto {
                dest,
                func_proto: func_name,
            } => {
                format!("%{} = FnProto {}", dest, func_name.to_string())
            }
        }
    }
}

// a terminator is a special instruction that ends a basic block,
// it can be a return, jump or branch instruction
#[derive(Debug, Clone)]
pub enum IRTerminator {
    // Returns from the function with the given operands
    Return(Vec<IROperand>),
    // Jump %label
    // unconditional jump to the given label
    Jump(usize),
    // Branch %cond, %then_label, %else_label
    // conditional branch based on %cond
    // if %cond is true, jump to %then_label
    // otherwise, %else_label
    Branch {
        cond: IROperand,
        br_true: usize,
        br_false: usize,
    },
    // FallThrough
    // no operation, just fall through to the next basic block
    FallThrough,
}

impl IRTerminator {
    pub fn to_string(&self) -> String {
        match self {
            IRTerminator::Return(operands) => {
                let ops_str = operands
                    .iter()
                    .map(|op| op.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Return [{}]", ops_str)
            }
            IRTerminator::Jump(label) => {
                format!("Jump _Tag{}", label)
            }
            IRTerminator::Branch {
                cond,
                br_true,
                br_false,
            } => {
                format!(
                    "Branch {}, _Tag{}, _Tag{}",
                    cond.to_string(),
                    br_true,
                    br_false
                )
            }
            IRTerminator::FallThrough => "FallThrough".to_string(),
        }
    }
}

// a basic block is a sequence of instructions
// that has only one entry point and one exit point
#[derive(Debug, Clone)]
pub struct IRBasicBlock {
    pub id: usize,
    pub instructions: Vec<IRInstruction>,
    pub terminator: IRTerminator,
}

impl IRBasicBlock {
    pub fn to_string(&self) -> String {
        let mut instrs_str = self
            .instructions
            .iter()
            .map(|instr| format!("  {}", instr.to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        let term_str = format!("  {}", self.terminator.to_string());
        instrs_str = if instrs_str.is_empty() {
            "".to_string()
        } else {
            format!("{}\n", instrs_str)
        };
        format!("_Tag{}:\n{}{}\n", self.id, instrs_str, term_str)
    }
}

#[derive(Debug, Clone)]
struct IRActiveBlock {
    pub id: usize,
    pub instructions: Vec<IRInstruction>,
}

// a function prototype, which is a template for function instances
// it contains the function name, parameters and body (basic blocks)
//
// as for the calling convention, the parameters are represented as the
// first N local variables of the function, where N is the number of parameters
// so if the function has 3 parameters, then the local variable slots 0, 1, 2 are used for the parameters
// if no return value is specified, the function will return a unit value
// you can handle this either by generating an nil or a unit value
// but there's no unit value in Lua spec, which is f*cking stupid
//
// by default, if a function is declared without a name,
// it will be treated as an anonymous function literal,
// and the prototype will be named with a generated unique name
// like __anon_fn_0, __anon_fn_1, etc.
//
// just pay attention, this is very important. - Li
#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub basic_blocks: Vec<IRBasicBlock>,
    pub local_variables: HashMap<String, IRLocalVarSlot>, // local variable name -> slot number
}

impl IRFunction {
    pub fn to_string(&self) -> String {
        let local_vars_str = if self.local_variables.is_empty() {
            "; <no local variables>".to_string()
        } else {
            let mut vars = self
                .local_variables
                .iter()
                .map(|(name, slot)| (slot, format!("; %local_{} = {}", slot, name)))
                .collect::<Vec<_>>();
            vars.sort_by_key(|(slot, _)| *slot);
            vars.iter()
                .map(|(_, s)| s.clone())
                .collect::<Vec<_>>()
                .join("\n")
        };

        let bbs_str = self
            .basic_blocks
            .iter()
            .map(|bb| bb.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let params = self
            .params
            .iter()
            .zip(0..)
            .map(|(p, i)| format!("param {}: %local_{}", p, i))
            .collect::<Vec<_>>();
        let param_str = if params.is_empty() {
            "void"
        } else {
            &params.join(", ")
        };
        format!(
            "function {}({}) {{\n{}\n{}}}",
            self.name, param_str, local_vars_str, bbs_str
        )
    }
}

#[derive(Debug, Clone)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
}

impl IRModule {
    pub fn to_string(&self) -> String {
        self.functions
            .iter()
            .map(|func| func.to_string())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum IRValueScope {
    Global,
    Local,
    UpVal,
}

impl IRGenerator {
    pub fn new() -> IRGenerator {
        return IRGenerator {
            module: IRModule { functions: vec![] },
            function_contexts: vec![],
            next_anonymous_func_id: 0,
            errors: vec![],
            scope_stack: vec![],
        };
    }

    pub fn get_err(&self) -> &Vec<IRGeneratorError> {
        &self.errors
    }

    fn current_context_mut(&mut self) -> &mut IRFunctionContext {
        self.function_contexts
            .last_mut()
            .expect("No active function context")
    }

    fn current_context(&self) -> &IRFunctionContext {
        self.function_contexts
            .last()
            .expect("No active function context")
    }

    fn emit(&mut self, instr: IRInstruction) {
        if let Some(active_block) = &mut self.current_context_mut().active_block {
            active_block.instructions.push(instr);
        } else {
            panic!("No active block to emit instruction");
        }
    }

    fn alloc_bb_id(&mut self) -> usize {
        let ctx = self.current_context_mut();
        let id = ctx.next_block_id;
        ctx.next_block_id += 1;
        id
    }

    fn alloc_anonymous_func_name(&mut self) -> String {
        let id = self.next_anonymous_func_id;
        self.next_anonymous_func_id += 1;
        format!("__anon_fn_{}", id)
    }

    fn open_bb(&mut self) -> usize {
        let id = self.alloc_bb_id();
        self.open_bb_lazy(id)
    }

    fn open_bb_lazy(&mut self, id: usize) -> usize {
        // this does not allocate a new block id
        self.current_context_mut().active_block = Some(IRActiveBlock {
            id,
            instructions: vec![],
        });
        id
    }

    fn close_bb(&mut self, terminator: IRTerminator) {
        let ctx = self.current_context_mut();
        if let Some(active_block) = ctx.active_block.take() {
            let bb = IRBasicBlock {
                id: active_block.id,
                instructions: active_block.instructions,
                terminator,
            };
            ctx.basic_blocks.push(bb);
        } else {
            panic!("No active block to close");
        }
    }

    fn has_active_bb(&self) -> bool {
        self.current_context().active_block.is_some()
    }

    fn open_function(&mut self, name: String, params: Vec<String>) {
        self.function_contexts.push(IRFunctionContext {
            name,
            params: params,
            active_block: None,
            basic_blocks: vec![],
            next_reg: 0,
            next_block_id: 0,
        });

        // enter the function scope
        self.push_scope();
    }

    fn close_function(&mut self) {
        // leave the function scope
        let local_vars = self
            .scope_stack
            .last()
            .expect("No active scope to get local variables")
            .local_variables
            .clone();

        self.pop_scope();

        let ctx = self
            .function_contexts
            .pop()
            .expect("No active function context");
        let func = IRFunction {
            name: ctx.name,
            params: ctx.params,
            basic_blocks: ctx.basic_blocks,
            local_variables: local_vars,
        };
        self.module.functions.push(func);
    }

    fn emit_err(&mut self, err: IRGeneratorError) {
        self.errors.push(err);
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(IRScope {
            local_variables: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    // declaring a local variable includes
    fn decl_local(&mut self, name: String) -> IRLocalVarSlot {
        if let Some(scope) = self.scope_stack.last_mut() {
            let slot = scope.local_variables.len();
            scope.local_variables.insert(name.clone(), slot);
            slot
        } else {
            panic!("No active scope to declare local variable");
        }
    }

    fn find_local(&self, name: &String) -> Option<IRLocalVarSlot> {
        if let Some(scope) = self.scope_stack.last() {
            return scope.local_variables.get(name).cloned();
        }
        None
    }

    fn var_scope(&self, name: &String) -> Option<IRValueScope> {
        // check local scopes from innermost to outermost
        let mut is_first = true;
        for scope in self.scope_stack.iter().rev() {
            if scope.local_variables.contains_key(name) {
                if is_first {
                    return Some(IRValueScope::Local);
                } else {
                    return Some(IRValueScope::UpVal);
                }
            }
            is_first = false;
        }

        // any undeclared variable is considered global
        Some(IRValueScope::Global)
    }

    fn alloc_reg(&mut self) -> usize {
        let ctx = self.current_context_mut();
        let reg = ctx.next_reg;
        ctx.next_reg += 1;
        reg
    }

    fn generate_assignment(
        &mut self,
        lhs: &parser::ast::Expression,
        rhs: &parser::ast::Expression,
    ) -> IROperand {
        let src = self.generate_expr(rhs);
        match lhs {
            parser::ast::Expression::Identifier(name) => {
                let scope = self.var_scope(name);
                match scope {
                    Some(IRValueScope::Local) => {
                        // local variable
                        // find the address (register) of the local variable
                        let slot = self.find_local(name);
                        if slot.is_none() {
                            self.emit_err(IRGeneratorError::UndefinedVariable(name.clone()));
                            return src;
                        }
                        let slot = slot.unwrap();

                        // get the address of the local variable by slot
                        let addr_reg = self.alloc_reg();
                        self.emit(IRInstruction::AddrLocal {
                            dest: addr_reg,
                            slot: IROperand::Slot(slot),
                        });

                        // store the value into the local variable
                        // assign the value to a new register and return it
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::StoreLocal {
                            dest: dest_reg,
                            dst: addr_reg,
                            src: src.clone(),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    Some(IRValueScope::Global) | None => {
                        // global variable
                        // if the variable is not declared, then also default to global
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::StoreGlobal {
                            dest: dest_reg,
                            name: name.clone(),
                            src: src.clone(),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    _ => {
                        self.emit_err(IRGeneratorError::UndefinedVariable(name.clone()));
                        unimplemented!("Assignment to undefined variable or upvalue");
                    }
                };
            }
            _ => {
                unimplemented!("Assignment to non-identifier left value");
            }
        }
    }

    fn generate_binary_expr(
        &mut self,
        op: &parser::ast::BinOp,
        left: &parser::ast::Expression,
        right: &parser::ast::Expression,
    ) -> IROperand {
        if let parser::ast::BinOp::Assign = op {
            return self.generate_assignment(left, right);
        }

        let left_reg = self.generate_expr(left);
        let right_reg = self.generate_expr(right);
        let dest_reg = self.alloc_reg();

        let ir_op = match op {
            parser::ast::BinOp::Add => IRBinOp::Add,
            parser::ast::BinOp::Sub => IRBinOp::Sub,
            parser::ast::BinOp::Mul => IRBinOp::Mul,
            parser::ast::BinOp::Div => IRBinOp::Div,
            parser::ast::BinOp::Pow => IRBinOp::Pow,
            parser::ast::BinOp::Concat => IRBinOp::Concat,
            parser::ast::BinOp::Eq => IRBinOp::Eq,
            parser::ast::BinOp::Neq => IRBinOp::Neq,
            parser::ast::BinOp::Lt => IRBinOp::Lt,
            parser::ast::BinOp::Gt => IRBinOp::Gt,
            parser::ast::BinOp::Leq => IRBinOp::Leq,
            parser::ast::BinOp::Geq => IRBinOp::Geq,
            parser::ast::BinOp::And => IRBinOp::And,
            parser::ast::BinOp::Or => IRBinOp::Or,
            parser::ast::BinOp::Assign => unreachable!(),
        };

        self.emit(IRInstruction::Binary {
            dest: dest_reg,
            src1: left_reg,
            src2: right_reg,
            operator: ir_op,
        });

        IROperand::Reg(dest_reg)
    }

    fn generate_unary_expr(
        &mut self,
        op: &parser::ast::UnOp,
        operand: &parser::ast::Expression,
    ) -> IROperand {
        let operand_reg = self.generate_expr(operand);
        let dest_reg = self.alloc_reg();

        let ir_op = match op {
            parser::ast::UnOp::Pos => unimplemented!(), // why do you need this sh*t?
            parser::ast::UnOp::Neg => IRUnOp::Neg,
            parser::ast::UnOp::Not => IRUnOp::Not,
        };

        self.emit(IRInstruction::Unary {
            dest: dest_reg,
            operator: ir_op,
            src: operand_reg,
        });

        IROperand::Reg(dest_reg)
    }

    fn generate_simple_literal(&mut self, lit: &parser::ast::Literal) -> IROperand {
        let imm_val = match lit {
            parser::ast::Literal::Number(n) => IROperand::ImmFloat(*n),
            parser::ast::Literal::String(s) => IROperand::ImmStr(s.clone()),
            parser::ast::Literal::Boolean(b) => IROperand::ImmBool(*b),
            parser::ast::Literal::Nil => IROperand::Nil,
            _ => {
                panic!("Not a simple literal");
            }
        };

        let dest_reg = self.alloc_reg();
        self.emit(IRInstruction::LoadImm {
            dest: dest_reg,
            value: imm_val,
        });
        IROperand::Reg(dest_reg)
    }

    fn generate_expr(&mut self, expr: &parser::ast::Expression) -> IROperand {
        match expr {
            parser::ast::Expression::Identifier(name) => {
                let scope = self.var_scope(name);
                match scope {
                    Some(IRValueScope::Local) => {
                        // local variable
                        // find the slot of the local variable
                        let slot = self.find_local(name).unwrap();

                        // get the address of the local variable by slot
                        let addr_reg = self.alloc_reg();
                        self.emit(IRInstruction::AddrLocal {
                            dest: addr_reg,
                            slot: IROperand::Slot(slot),
                        });

                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadLocal {
                            dest: dest_reg,
                            src: IROperand::Reg(addr_reg),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    Some(IRValueScope::Global) | None => {
                        // global variable
                        // if the variable is not declared, then also default to global load
                        // however this can fail at runtime if the variable is not defined
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadGlobal {
                            dest: dest_reg,
                            name: name.clone(),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    _ => {
                        self.emit_err(IRGeneratorError::UndefinedVariable(name.clone()));
                        return IROperand::Nil;
                    }
                };
            }
            parser::ast::Expression::Literal(lit) => match lit {
                parser::ast::Literal::Number(_)
                | parser::ast::Literal::String(_)
                | parser::ast::Literal::Boolean(_)
                | parser::ast::Literal::Nil => {
                    return self.generate_simple_literal(lit);
                }
                parser::ast::Literal::Function { name, params, body } => {
                    // function literal
                    // this generates a function prototype and returns the function reference
                    let func_operand = self.generate_fn_decl_impl(true, name, params, body);

                    // instantiate the function prototype
                    let dest_reg = self.alloc_reg();
                    self.emit(IRInstruction::FnProto {
                        dest: dest_reg,
                        func_proto: func_operand,
                    });

                    let func_operand = IROperand::Reg(dest_reg);
                    return func_operand;
                }
            },
            parser::ast::Expression::BinOp {
                left,
                operator,
                right,
            } => {
                return self.generate_binary_expr(operator, left, right);
            }
            parser::ast::Expression::UnOp { operator, operand } => {
                return self.generate_unary_expr(operator, operand);
            }
            parser::ast::Expression::FnCall { callee, arguments } => {
                // any fn
                let callee_reg = self.generate_expr(callee);
                // args
                let mut arg_regs = vec![];
                for arg in arguments {
                    let arg_reg = self.generate_expr(arg);
                    arg_regs.push(arg_reg);
                }

                let dest_reg = self.alloc_reg();
                self.emit(IRInstruction::Call {
                    dest: dest_reg,
                    callee: callee_reg,
                    args: arg_regs,
                });

                return IROperand::Reg(dest_reg);
            }
            parser::ast::Expression::IndexOf { collection, index } => {
                // collection and index
                let collection_reg = self.generate_expr(collection);
                let index_reg = self.generate_expr(index);

                let dest_reg = self.alloc_reg();
                self.emit(IRInstruction::IndexOf {
                    dest: dest_reg,
                    collection: collection_reg,
                    index: index_reg,
                });
                return IROperand::Reg(dest_reg);
            }
        }
    }

    fn generate_if_expr(
        &mut self,
        condition: &parser::ast::Expression,
        then_branch: &Vec<parser::ast::Statement>,
        else_branch: &Option<Vec<parser::ast::Statement>>,
    ) {
        let cond_reg = self.generate_expr(condition);

        // then, else blocks
        // todo: elif branches
        let then_bb_id = self.alloc_bb_id();
        let else_bb_id = self.alloc_bb_id();
        let merge_bb_id = self.alloc_bb_id();

        self.close_bb(IRTerminator::Branch {
            cond: cond_reg,
            br_true: then_bb_id,
            br_false: else_bb_id,
        });

        self.open_bb_lazy(then_bb_id);
        for stmt in then_branch {
            self.generate_stmt(stmt);
        }
        self.close_bb(IRTerminator::Jump(merge_bb_id));

        self.open_bb_lazy(else_bb_id);
        if let Some(else_branch) = else_branch {
            for stmt in else_branch {
                self.generate_stmt(stmt);
            }
        }
        self.close_bb(IRTerminator::Jump(merge_bb_id));

        self.open_bb_lazy(merge_bb_id);
    }

    fn generate_while_expr(
        &mut self,
        condition: &parser::ast::Expression,
        body: &Vec<parser::ast::Statement>,
    ) {
        let cond_bb_id = self.alloc_bb_id();
        let body_bb_id = self.alloc_bb_id();
        let merge_bb_id = self.alloc_bb_id();

        // fall through to condition check first
        self.close_bb(IRTerminator::FallThrough);

        // condition check block
        self.open_bb_lazy(cond_bb_id);
        let cond_reg = self.generate_expr(condition);
        self.close_bb(IRTerminator::Branch {
            cond: cond_reg,
            br_true: body_bb_id,
            br_false: merge_bb_id,
        });

        // loop body block
        self.open_bb_lazy(body_bb_id);
        for stmt in body {
            self.generate_stmt(stmt);
        }
        // after body, jump back to condition check
        self.close_bb(IRTerminator::Jump(cond_bb_id));

        // merge block
        self.open_bb_lazy(merge_bb_id);
    }

    fn generate_repeat_expr(
        &mut self,
        body: &Vec<parser::ast::Statement>,
        condition: &parser::ast::Expression,
    ) {
        let body_bb_id = self.alloc_bb_id();
        let cond_bb_id = self.alloc_bb_id();
        let merge_bb_id = self.alloc_bb_id();

        // fall through to loop body first
        self.close_bb(IRTerminator::FallThrough);

        // loop body block
        self.open_bb_lazy(body_bb_id);
        for stmt in body {
            self.generate_stmt(stmt);
        }
        // after body, fall through to condition check
        self.close_bb(IRTerminator::FallThrough);

        // condition check block
        self.open_bb_lazy(cond_bb_id);
        let cond_reg = self.generate_expr(condition);
        self.close_bb(IRTerminator::Branch {
            cond: cond_reg,
            br_true: merge_bb_id,
            br_false: body_bb_id,
        });

        // merge block
        self.open_bb_lazy(merge_bb_id);
    }

    fn generate_fn_decl_impl(
        &mut self,
        is_local: bool,
        name: &Option<String>,
        params: &Vec<String>,
        body: &Vec<parser::ast::Statement>,
    ) -> IROperand {
        let func_name = if let Some(name) = name {
            name.clone()
        } else {
            self.alloc_anonymous_func_name()
        };

        // create a new function context
        self.open_function(func_name.clone(), params.clone());

        // declare parameters as local variables
        for param in params {
            self.decl_local(param.clone());
        }

        // generate function body
        self.open_bb();
        for stmt in body {
            self.generate_stmt(stmt);
        }

        // if the block is still open, close it with a return
        if self.has_active_bb() {
            self.close_bb(IRTerminator::Return(vec![IROperand::Unit]));
        }

        self.close_function();

        // return the function prototype as an operand
        IROperand::Proto(func_name)
    }

    fn generate_return_stmt(&mut self, values: &Vec<parser::ast::Expression>) {
        let mut ret_operands = vec![];
        for val in values {
            let val_reg = self.generate_expr(val);
            ret_operands.push(val_reg);
        }
        self.close_bb(IRTerminator::Return(ret_operands));
    }

    fn generate_stmt(&mut self, stmt: &parser::ast::Statement) {
        match stmt {
            parser::ast::Statement::ExprStatement(expr) => {
                let reg = self.generate_expr(expr);
                // drop the result of the expression statement, since not used
                self.emit(IRInstruction::Drop { src: reg });
            }
            parser::ast::Statement::Declaration { names, values } => {
                for (name, value) in names.iter().zip(values.iter()) {
                    let src = self.generate_expr(value);
                    // by default, 'Declaration' is for local variables
                    let scope = self.var_scope(name);
                    let slot = if scope == Some(IRValueScope::Local) {
                        // redefinition of local variable in the same scope
                        // this can happen, for example, in:
                        // if cond then
                        //     local x = 1
                        // else
                        //     local x = 2 -- redefinition in the same scope
                        // end
                        //
                        // if already declared, just use the existing register
                        self.find_local(name).unwrap()
                    } else {
                        // otherwise, declare a new local variable
                        self.decl_local(name.clone())
                    };

                    let addr_reg = self.alloc_reg();
                    self.emit(IRInstruction::AddrLocal {
                        dest: addr_reg,
                        slot: IROperand::Slot(slot),
                    });

                    let dest_reg = self.alloc_reg();
                    self.emit(IRInstruction::StoreLocal {
                        dest: dest_reg,
                        dst: addr_reg,
                        src: src.clone(),
                    });

                    // StoreLocal returns the value stored, but we don't need it for declaration
                    // drop it
                    self.emit(IRInstruction::Drop {
                        src: IROperand::Reg(dest_reg),
                    });
                }
            }
            parser::ast::Statement::IfStmt {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            } => {
                self.generate_if_expr(condition, then_branch, else_branch);
            }
            parser::ast::Statement::WhileStmt { condition, body } => {
                self.generate_while_expr(condition, body);
            }
            parser::ast::Statement::RepeatStmt { body, condition } => {
                self.generate_repeat_expr(body, condition);
            }
            parser::ast::Statement::ReturnStmt { values } => {
                self.generate_return_stmt(values);
            }
            _ => unimplemented!(),
        }
    }

    fn generate_module(&mut self, module: &parser::ast::Program) {
        // 'global' local scope
        // we just put the whole program in a special function named "_start"
        // for top level stmts
        self.open_function("_start".to_string(), vec![]);

        self.open_bb();
        for stmt in &module.body {
            self.generate_stmt(stmt);
        }

        // if the block is still open, close it with a return
        if self.has_active_bb() {
            self.close_bb(IRTerminator::Return(vec![IROperand::Unit]));
        }

        self.close_function();
    }

    pub fn generate(&mut self, program: &parser::ast::Program) {
        self.generate_module(program);
    }

    pub fn get_module(&self) -> &IRModule {
        &self.module
    }
}

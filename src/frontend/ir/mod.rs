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
//      26-02-12: Added LoadImm instruction for loading immediate values
//      26-02-12: Added Drop instruction for discarding values that are not needed
//      26-02-12: Now *Global instructions require a register to hold the var name,
//                instead of directly using the name as an operand,
//      26-02-13: Added TableCtor expression support in IR generation
//      26-02-13: Added member access support in IR generation
//      26-02-14: Removed AddrLocal instruction, replaced with direct use of local variable slots
//                in LoadLocal and StoreLocal, to simplify the IR and avoid unnecessary instructions
//      26-02-14: Added mangling for local function names to avoid name conflicts
//      26-02-18: Refactored IR generator and removed some redundant features like scope stack
//      26-02-18: Added subfunction metadata for functions
//      26-02-19: Fixed a bug in handling of early returns in if statements and loops, 
//                now it will try to close the current basic block only when a block is active,
//                instead of unconditionally closing a block, which may panic
//      26-02-20: UpVal analysis and handling in IR generation

use std::collections::{HashMap};

use crate::frontend::parser;

pub struct IRGenerator {
    module: IRModule,
    function_contexts: Vec<IRFunctionContext>,

    next_func_id: usize,

    errors: Vec<IRGeneratorError>,
}

type IRLocalVarSlot = usize;
type IRUpValSlot = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum IRUpValType {
    LocalVar(usize), // the slot number of the local variable captured of the parent function
    UpVal(usize),    // the index of the upvalue in the parent function's upvalue list
}

#[derive(Debug, Clone, PartialEq)]
pub struct IRUpVal {
    slot: IRUpValSlot,
    ty: IRUpValType,
}

#[derive(Debug, Clone)]
struct IRFunctionContext {
    name: String,
    params: Vec<String>,

    // local variable name -> slot number
    local_variables: HashMap<String, IRLocalVarSlot>,
    upvalues: HashMap<String, IRUpVal>,

    // names of sub function prototypes
    sub_functions: Vec<String>,

    active_block: Option<IRActiveBlock>,
    basic_blocks: Vec<IRBasicBlock>,

    next_reg: usize,
    next_block_id: usize,
}

#[derive(Debug, Clone)]
pub enum IRGeneratorError {
    UndefinedVariable(String),
    InvalidLValue,
    MultipleReturnStatements,
}

#[derive(Debug, Clone)]
pub enum IROperand {
    // virtual register
    Reg(usize),
    // Local variable slot, used for *Local instructions,
    // DO NOT use this directly as an operand in any other instructions
    Slot(IRLocalVarSlot),
    // Upval slot
    UpVal(IRUpValSlot),
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
            IROperand::UpVal(slot) => format!("%upval_{}", slot),
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
    // %dest = LoadLocal %slot
    // load the value of local variable at slot %slot into %dest
    // %slot is guaranteed to be a IROperand::Slot
    //
    // see: IRFunction::local_variables for mapping information
    //      of local variable names to slot numbers
    LoadLocal {
        dest: usize,
        src: IROperand,
    },
    // StoreLocal %slot, %src
    // store the value of %src into local variable at slot %slot
    // returns the value stored, which is useful for chained assignments like a = b = c
    // %slot is guaranteed to be a IROperand::Slot
    //
    // see: IRFunction::local_variables for mapping information
    //      of local variable names to slot numbers
    StoreLocal {
        dest: usize,
        dst: IROperand,
        src: IROperand,
    },
    // %dest = LoadGlobal "name"
    // load the value of global variable "name" into %dest
    LoadGlobal {
        dest: usize,
        name: IROperand,
    },
    // StoreGlobal "name" %src
    // store the value of %src into global variable "name"
    // returns the value stored, same as StoreLocal, for chained assignment support
    StoreGlobal {
        dest: usize,
        name: IROperand,
        src: IROperand,
    },
    // %dest = LoadUpVal %upval_slot
    // load the value of upvalue at slot %upval_slot into %dest
    // %upval_slot is guaranteed to be a IROperand::UpVal
    LoadUpVal {
        dest: usize,
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
    // This is basically a fast path for table member access,
    // i.e. GetTable with a constant key
    // but whether to implement this instruction is optional,
    // backend can choose to lower this to GetTable
    IndexOf {
        dest: usize,
        collection: IROperand,
        index: IROperand,
    },
    // %dest = SetIndex %collection, %index, %value
    // Set the element at %index in %collection to %value,
    // a fast path for table member assignment, similar to IndexOf
    SetIndex {
        dest: usize,
        collection: IROperand,
        index: IROperand,
        value: IROperand,
    },
    // %dest = MemberOf %collection, %member
    // Get the member %member from %collection,
    // %member is guaranteed to be a string literal,
    // so this is also a fast path for table member access,
    // you can precompute the hash of the member name and use it as a key in GetTable,
    // but again, whether to implement this instruction is optional
    MemberOf {
        dest: usize,
        collection: IROperand,
        member: IROperand,
    },
    // %dest = SetMember %collection, %member, %value
    // Set the member %member in %collection to %value,
    // similar to MemberOf, this is a fast path for table member assignment
    // %member is guaranteed to be a string literal, same as in MemberOf
    SetMember {
        dest: usize,
        collection: IROperand,
        member: IROperand,
        value: IROperand,
    },
    // %dest = NewTable %size_array, %size_hash
    // Create a new table with preallocated sizes
    // store the table reference into %dest
    // However the preallocation behavior is implementation-defined,
    // so it just serves as a hint for optimization
    //
    // size_array: expected number of array-like elements, if applicable
    // size_hash:  expected number of key-value pairs, if applicable
    // this can be mixed, so a table can contain both array-like and hash-like elements
    //
    // Something worth noticing for registers holding table references is that,
    // the table it points to is mutable, while the register itself is immutable,
    // which means that you can modify the contents of the table through that only register
    // and this register can survive multiple instructions
    // so you cannot simply discard the register after one use
    NewTable {
        dest: usize,
        size_array: IROperand,
        size_hash: IROperand,
    },
    // %dest = SetTable %table, %key, %value
    // Set the key-value pair in the table
    // This instruction returns the value set, for chained assignments
    SetTable {
        dest: usize,
        table: IROperand,
        key: IROperand,
        value: IROperand,
    },
    // %dest = GetTable %table, %key
    // Get the value from the table at the given key
    // This is the most general form of table access, which can handle any type of key,
    GetTable {
        dest: usize,
        table: IROperand,
        key: IROperand,
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
            IRInstruction::LoadLocal { dest, src } => {
                format!("%{} = LoadLocal {}", dest, src.to_string())
            }
            IRInstruction::StoreLocal { dest, dst, src } => {
                format!(
                    "%{} = StoreLocal {} {}",
                    dest,
                    dst.to_string(),
                    src.to_string()
                )
            }
            IRInstruction::LoadGlobal { dest, name } => {
                format!("%{} = LoadGlobal {}", dest, name.to_string())
            }
            IRInstruction::StoreGlobal { dest, name, src } => {
                format!(
                    "%{} = StoreGlobal {} {}",
                    dest,
                    name.to_string(),
                    src.to_string()
                )
            }
            IRInstruction::LoadUpVal { dest, src } => {
                format!("%{} = LoadUpVal {}", dest, src.to_string())
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
            IRInstruction::SetIndex {
                dest,
                collection,
                index,
                value,
            } => {
                format!(
                    "%{} = SetIndex {}, {}, {}",
                    dest,
                    collection.to_string(),
                    index.to_string(),
                    value.to_string()
                )
            }
            IRInstruction::MemberOf {
                dest,
                collection,
                member,
            } => {
                format!(
                    "%{} = MemberOf {}, {}",
                    dest,
                    collection.to_string(),
                    member.to_string()
                )
            }
            IRInstruction::SetMember {
                dest,
                collection,
                member,
                value,
            } => {
                format!(
                    "%{} = SetMember {}, {}, {}",
                    dest,
                    collection.to_string(),
                    member.to_string(),
                    value.to_string()
                )
            }
            IRInstruction::NewTable {
                dest,
                size_array,
                size_hash,
            } => {
                format!(
                    "%{} = NewTable {}, {}",
                    dest,
                    size_array.to_string(),
                    size_hash.to_string()
                )
            }
            IRInstruction::SetTable {
                dest,
                table,
                key,
                value,
            } => {
                format!(
                    "%{} = SetTable {}, {}, {}",
                    dest,
                    table.to_string(),
                    key.to_string(),
                    value.to_string()
                )
            }
            IRInstruction::GetTable { dest, table, key } => {
                format!(
                    "%{} = GetTable {}, {}",
                    dest,
                    table.to_string(),
                    key.to_string()
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
    pub upvalues: HashMap<String, IRUpVal>, // upvalue name -> upvalue info
    pub sub_functions: Vec<String>, // names of sub function prototypes
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

        let upvals_str = if self.upvalues.is_empty() {
            "; <no upvalues>".to_string()
        } else {
            let mut upvals = self
                .upvalues
                .iter()
                .map(|(name, upval)| {
                    let ty_str = match &upval.ty {
                        IRUpValType::LocalVar(slot) => format!("%local_{} of parent", slot),
                        IRUpValType::UpVal(slot) => format!("%upval_{} of parent", slot),
                    };
                    format!("; %upval_{} = {} ({})", upval.slot, name, ty_str)
                })
                .collect::<Vec<_>>();
            upvals.sort_by_key(|s| s.clone());
            upvals.join("\n")
        };

        let sub_fns_str = if self.sub_functions.is_empty() {
            "; <no sub functions>".to_string()
        } else {
            self.sub_functions
                .iter()
                .zip(0..)
                .map(|(name, i)| format!("; subfn #{}: @{}", i, name))
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
            "function {}({}) {{\n{}\n;\n{}\n;\n{}\n{}}}",
            self.name, param_str, local_vars_str, upvals_str, sub_fns_str, bbs_str
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
    Local(usize),
    UpVal(IRUpVal),
}

impl IRGenerator {
    pub fn new() -> IRGenerator {
        return IRGenerator {
            module: IRModule { functions: vec![] },
            function_contexts: vec![],
            next_func_id: 0,
            errors: vec![],
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
        // generate a unique name for anonymous function literals
        let id = self.next_func_id;
        self.next_func_id += 1;
        format!("__anon_fn_{}", id)
    }

    fn mangle_local_fn_name(&mut self, name: &String) -> String {
        // mangle the local function name to avoid name conflicts
        // "foo" -> "__local_fn_foo_0", "__local_fn_foo_1", etc.
        let id = self.next_func_id;
        self.next_func_id += 1;
        format!("__local_fn_{}_{}", name, id)
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

    // try to close a possible active basic block with the given terminator,
    // if there are no active basic block, it will do nothing instead of panicking,
    // this is intended for specific scenarios, e.g. early return
    //
    // if x > 0 then 
    //    return x
    // else
    //    return -x
    // end
    //
    // if we just use close_bb, the program will crash
    fn try_close_bb(&mut self, terminator: IRTerminator) {
        if self.has_active_bb() {
            self.close_bb(terminator);
        }
    }

    fn has_active_bb(&self) -> bool {
        self.current_context().active_block.is_some()
    }

    fn open_function(&mut self, name: String, params: Vec<String>) {
        // if not topmost,
        // add sub function name to sub function list of the current function
        if let Some(ctx) = self.function_contexts.last_mut() {
            ctx.sub_functions.push(name.clone());
        }

        self.function_contexts.push(IRFunctionContext {
            name,
            params: params,
            local_variables: HashMap::new(),
            upvalues: HashMap::new(),
            sub_functions: vec![],
            active_block: None,
            basic_blocks: vec![],
            next_reg: 0,
            next_block_id: 0,
        });
    }

    fn close_function(&mut self) {
        // leave the function scope
        let local_vars = self
            .current_context()
            .local_variables
            .clone();

        let ctx = self
            .function_contexts
            .pop()
            .expect("No active function context");
        let func = IRFunction {
            name: ctx.name,
            params: ctx.params,
            basic_blocks: ctx.basic_blocks,
            local_variables: local_vars,
            upvalues: ctx.upvalues,
            sub_functions: ctx.sub_functions,
        };
        self.module.functions.push(func);
    }

    fn emit_err(&mut self, err: IRGeneratorError) {
        self.errors.push(err);
    }

    // declaring a local variable includes
    fn decl_local(&mut self, name: String) -> IRLocalVarSlot {
        let slot = self.current_context_mut().local_variables.len();
        self.current_context_mut().local_variables.insert(name.clone(), slot);
        slot
    }

    fn find_local(&self, name: &String) -> Option<IRLocalVarSlot> {
        self.current_context()
            .local_variables
            .get(name)
            .cloned()
    }

    fn add_upval_to_context(&mut self, func_idx: usize, name: &String, ty: IRUpValType) -> IRUpVal {
        let ctx = &mut self.function_contexts[func_idx];
        let slot = ctx.upvalues.len();
        let uv = IRUpVal { slot, ty };
        ctx.upvalues.insert(name.clone(), uv.clone());
        uv
    }

    fn var_scope_impl(&mut self, func_idx: usize, name: &String) -> Option<IRValueScope> {
        let current_context = &self.function_contexts[func_idx];

        if let Some(&slot) = current_context.local_variables.get(name) {
            return Some(IRValueScope::Local(slot));
        }

        if let Some(uv) = current_context.upvalues.get(name) {
            return Some(IRValueScope::UpVal(uv.clone()));
        }

        if func_idx > 0 {
            if let Some(parent_scope) = self.var_scope_impl(func_idx - 1, name) {
                match parent_scope {
                    IRValueScope::Local(slot) => {
                        let uv = self.add_upval_to_context(func_idx, name, IRUpValType::LocalVar(slot));
                        return Some(IRValueScope::UpVal(uv));
                    }
                    IRValueScope::UpVal(parent_uv) => {
                        let uv = self.add_upval_to_context(func_idx, name, IRUpValType::UpVal(parent_uv.slot));
                        return Some(IRValueScope::UpVal(uv));
                    }
                    IRValueScope::Global => {
                        // global variable, do nothing
                    }
                }
            }
        }

        Some(IRValueScope::Global)
    }

    fn var_scope(&mut self, name: &String) -> Option<IRValueScope> {
        let current_func_idx = self.function_contexts.len() - 1;
        self.var_scope_impl(current_func_idx, name)
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
                    Some(IRValueScope::Local(slot)) => {
                        // local variable
                        // store the value into the local variable
                        // assign the value to a new register and return it
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::StoreLocal {
                            dest: dest_reg,
                            dst: IROperand::Slot(slot),
                            src: src.clone(),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    Some(IRValueScope::Global) | None => {
                        // global variable
                        // first generate a LoadConst instruction to load the global variable name
                        let name_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadImm {
                            dest: name_reg,
                            value: IROperand::ImmStr(name.clone()),
                        });

                        // if the variable is not declared, then also default to global
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::StoreGlobal {
                            dest: dest_reg,
                            name: IROperand::Reg(name_reg),
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
            // non-trivial lvalue, like table member access or indexing
            parser::ast::Expression::MemberAccess { collection, member } => {
                // generate code for table member assignment
                // here for x.y.z.w, we unwrap it to (x.y.z) and w,
                // and generate code for getting the table reference of x.y.z first,
                // then set the member w in that table
                let collection_reg = self.generate_expr(collection);

                let key_reg = self.alloc_reg();
                self.emit(IRInstruction::LoadImm {
                    dest: key_reg,
                    value: IROperand::ImmStr(member.clone()),
                });

                let dest_reg = self.alloc_reg();
                self.emit(IRInstruction::SetMember {
                    dest: dest_reg,
                    collection: collection_reg,
                    member: IROperand::Reg(key_reg),
                    value: src.clone(),
                });
                return IROperand::Reg(dest_reg);
            }
            parser::ast::Expression::IndexOf { collection, index } => {
                // collection and index
                // this is similar to that in generate_expr,
                // but we need setter instructions instead of getter instructions
                let collection_reg = self.generate_expr(collection);

                match &**index {
                    parser::ast::Expression::Literal(parser::ast::Literal::String(s)) => {
                        // string literal key
                        let key_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadImm {
                            dest: key_reg,
                            value: IROperand::ImmStr(s.clone()),
                        });

                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::SetMember {
                            dest: dest_reg,
                            collection: collection_reg,
                            member: IROperand::Reg(key_reg),
                            value: src.clone(),
                        });
                        IROperand::Reg(dest_reg)
                    }
                    parser::ast::Expression::Literal(parser::ast::Literal::Number(n)) => {
                        // numeric index
                        let key_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadImm {
                            dest: key_reg,
                            value: IROperand::ImmFloat(*n),
                        });

                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::SetIndex {
                            dest: dest_reg,
                            collection: collection_reg,
                            index: IROperand::Reg(key_reg),
                            value: src.clone(),
                        });
                        IROperand::Reg(dest_reg)
                    }
                    _ => {
                        // general case
                        let key_reg = self.generate_expr(index);
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::SetTable {
                            dest: dest_reg,
                            table: collection_reg,
                            key: key_reg,
                            value: src.clone(),
                        });
                        IROperand::Reg(dest_reg)
                    }
                }
            }
            _ => {
                self.emit_err(IRGeneratorError::InvalidLValue);
                src
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

    fn generate_table_ctor_expr(
        &mut self,
        fields: &Vec<(Option<parser::ast::Expression>, parser::ast::Expression)>,
    ) -> IROperand {
        // make table prototype

        // make sizes
        let (asize, hsize) = {
            // count array-like and hash-like fields
            let (asize, hsize) = fields.iter().fold((0, 0), |(a, h), (key_opt, _)| {
                if key_opt.is_none() {
                    (a + 1, h)
                } else {
                    (a, h + 1)
                }
            });
            let asize_op = IROperand::ImmFloat(asize as f64);
            let hsize_op = IROperand::ImmFloat(hsize as f64);

            let asize_reg = self.alloc_reg();
            self.emit(IRInstruction::LoadImm {
                dest: asize_reg,
                value: asize_op,
            });

            let hsize_reg = self.alloc_reg();
            self.emit(IRInstruction::LoadImm {
                dest: hsize_reg,
                value: hsize_op,
            });

            (IROperand::Reg(asize_reg), IROperand::Reg(hsize_reg))
        };

        // create table register
        let tbl_reg = self.alloc_reg();
        self.emit(IRInstruction::NewTable {
            dest: tbl_reg,
            size_array: asize,
            size_hash: hsize,
        });

        let tbl_reg = IROperand::Reg(tbl_reg);

        // set fields
        // lua tables are 1-indexed!!!
        let mut idx = 1;

        fields.iter().for_each(|(key_opt, value_expr)| {
            match key_opt {
                Some(k) => {
                    // hash-like, use provided key
                    // two cases: { [expr] = y, ... } or { x = y, ... }
                    // for the latter, key_expr is preprocessed into a string literal expression
                    // so we can just generate the expression directly
                    // and, there's possibly manually specified num literal indices
                    //
                    // see: Parser::parse_table_ctor for details
                    let key_reg = self.generate_expr(k);
                    let value_reg = self.generate_expr(value_expr);
                    let dest_reg = self.alloc_reg();
                    match k {
                        parser::ast::Expression::Literal(parser::ast::Literal::String(_)) => {
                            // string literal key, can use SetMember instruction
                            self.emit(IRInstruction::SetMember {
                                dest: dest_reg,
                                collection: tbl_reg.clone(),
                                member: key_reg,
                                value: value_reg,
                            });
                        }
                        parser::ast::Expression::Literal(parser::ast::Literal::Number(_)) => {
                            // numeric key, can use IndexOf instruction
                            // this is basically array-like access, but with explicit keys
                            self.emit(IRInstruction::SetIndex {
                                dest: dest_reg,
                                collection: tbl_reg.clone(),
                                index: key_reg,
                                value: value_reg,
                            });
                        }
                        _ => {
                            // general case, use SetTable with generated key operand
                            self.emit(IRInstruction::SetTable {
                                dest: dest_reg,
                                table: tbl_reg.clone(),
                                key: key_reg,
                                value: value_reg,
                            });
                        }
                    }
                }
                None => {
                    // array-like field, key is the next index
                    // we can directly use the index as the key operand in SetIndex
                    let key_reg = self.alloc_reg();
                    self.emit(IRInstruction::LoadImm {
                        dest: key_reg,
                        value: IROperand::ImmFloat(idx as f64),
                    });

                    let value_reg = self.generate_expr(value_expr);

                    let dest_reg = self.alloc_reg();
                    self.emit(IRInstruction::SetIndex {
                        dest: dest_reg,
                        collection: tbl_reg.clone(),
                        index: IROperand::Reg(key_reg),
                        value: value_reg,
                    });
                }
            }
            idx += 1;
        });

        tbl_reg
    }

    fn generate_expr(&mut self, expr: &parser::ast::Expression) -> IROperand {
        match expr {
            parser::ast::Expression::Identifier(name) => {
                let scope = self.var_scope(name);
                match scope {
                    Some(IRValueScope::Local(slot)) => {
                        // local variable
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadLocal {
                            dest: dest_reg,
                            src: IROperand::Slot(slot),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    Some(IRValueScope::Global) | None => {
                        // global variable
                        // if the variable is not declared, then also default to global load
                        // however this can fail at runtime if the variable is not defined

                        // load name
                        let name_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadImm {
                            dest: name_reg,
                            value: IROperand::ImmStr(name.clone()),
                        });

                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadGlobal {
                            dest: dest_reg,
                            name: IROperand::Reg(name_reg),
                        });
                        return IROperand::Reg(dest_reg);
                    }
                    Some(IRValueScope::UpVal(upval)) => {
                        // upval
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::LoadUpVal {
                            dest: dest_reg,
                            src: IROperand::UpVal(upval.slot),
                        });
                        return IROperand::Reg(dest_reg);
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
            } => self.generate_binary_expr(operator, left, right),
            parser::ast::Expression::UnOp { operator, operand } => {
                self.generate_unary_expr(operator, operand)
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

                IROperand::Reg(dest_reg)
            }
            parser::ast::Expression::IndexOf { collection, index } => {
                // collection and index
                // this has few types of possibilites:
                // 1. common table access like t[k], where collection is a table and index is any expression
                // 2. table access with string literal key
                // 3. table access with numeric index, which is basically array access
                let collection_reg = self.generate_expr(collection);
                let index_reg = self.generate_expr(index);

                match **index {
                    parser::ast::Expression::Literal(parser::ast::Literal::String(_)) => {
                        // string literal key, can use MemberOf instruction
                        // if backend implements MemberOf, it can prehash the member name
                        // and optimize the access
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::MemberOf {
                            dest: dest_reg,
                            collection: collection_reg,
                            member: index_reg,
                        });
                        IROperand::Reg(dest_reg)
                    }
                    parser::ast::Expression::Literal(parser::ast::Literal::Number(_)) => {
                        // numeric index, can use IndexOf instruction
                        // if backend implements IndexOf, this can be optimized as array access
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::IndexOf {
                            dest: dest_reg,
                            collection: collection_reg,
                            index: index_reg,
                        });
                        IROperand::Reg(dest_reg)
                    }
                    _ => {
                        // general case, use GetTable instruction
                        let dest_reg = self.alloc_reg();
                        self.emit(IRInstruction::GetTable {
                            dest: dest_reg,
                            table: collection_reg,
                            key: index_reg,
                        });
                        IROperand::Reg(dest_reg)
                    }
                }
            }
            parser::ast::Expression::MemberAccess { collection, member } => {
                // normal member access like t.x, which is syntactic sugar for t["x"]
                let collection_reg = self.generate_expr(collection);

                let member_reg = self.alloc_reg();
                self.emit(IRInstruction::LoadImm {
                    dest: member_reg,
                    value: IROperand::ImmStr(member.clone()),
                });

                let dest_reg = self.alloc_reg();
                self.emit(IRInstruction::MemberOf {
                    dest: dest_reg,
                    collection: collection_reg,
                    member: IROperand::Reg(member_reg),
                });
                IROperand::Reg(dest_reg)
            }
            parser::ast::Expression::TableCtor { fields } => self.generate_table_ctor_expr(fields),
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

        self.try_close_bb(IRTerminator::Branch {
            cond: cond_reg,
            br_true: then_bb_id,
            br_false: else_bb_id,
        });

        self.open_bb_lazy(then_bb_id);
        for stmt in then_branch {
            self.generate_stmt(stmt);
        }
        self.try_close_bb(IRTerminator::Jump(merge_bb_id));

        self.open_bb_lazy(else_bb_id);
        if let Some(else_branch) = else_branch {
            for stmt in else_branch {
                self.generate_stmt(stmt);
            }
        }
        self.try_close_bb(IRTerminator::Jump(merge_bb_id));

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
        self.try_close_bb(IRTerminator::FallThrough);

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
        self.try_close_bb(IRTerminator::Jump(cond_bb_id));

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
        self.try_close_bb(IRTerminator::FallThrough);

        // loop body block
        self.open_bb_lazy(body_bb_id);
        for stmt in body {
            self.generate_stmt(stmt);
        }
        // after body, fall through to condition check
        self.try_close_bb(IRTerminator::FallThrough);

        // condition check block
        self.open_bb_lazy(cond_bb_id);
        let cond_reg = self.generate_expr(condition);
        self.try_close_bb(IRTerminator::Branch {
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
            if is_local {
                // if local, mangle it to avoid name conflicts
                self.mangle_local_fn_name(name)
            } else {
                // if global, just use the name directly
                // note that the behavior of overwriting is not properly defined
                // in this type of implementation
                // so we'd better avoid any global function, except for entry point
                name.clone()
            }
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
        self.try_close_bb(IRTerminator::Return(vec![IROperand::Unit]));

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

        // this should be the last instruction in the current basic block
        if !self.has_active_bb() {
            self.emit_err(IRGeneratorError::MultipleReturnStatements);
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
                    let scope = self.find_local(name);
                    let slot = if let Some(slot) = scope {
                        // redefinition of local variable in the same scope
                        // this can happen, for example, in:
                        // if cond then
                        //     local x = 1
                        // else
                        //     local x = 2 -- redefinition in the same scope
                        // end
                        //
                        // if already declared, just use the existing register
                        slot
                    } else {
                        // otherwise, declare a new local variable
                        self.decl_local(name.clone())
                    };

                    let dest_reg = self.alloc_reg();
                    self.emit(IRInstruction::StoreLocal {
                        dest: dest_reg,
                        dst: IROperand::Slot(slot),
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
                if !elif_branches.is_empty() {
                    unimplemented!("Elif branches are not supported yet");
                }
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
        self.try_close_bb(IRTerminator::Return(vec![IROperand::Unit]));

        self.close_function();
    }

    pub fn generate(&mut self, program: &parser::ast::Program) {
        self.generate_module(program);
    }

    pub fn get_module(&self) -> &IRModule {
        &self.module
    }
}

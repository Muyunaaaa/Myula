// Myula compiler VM
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-02-15: Finalized the VM data structures and core components;
//            designed FuncMetadata to store function-related bytecode, constant pools, and register lifetime information;
//            implemented the VirtualMachine initialization process, including function extraction from IR modules, bytecode generation, and entry frame preparation;
//            introduced the dump_internal_state method for debugging and verifying VM state;
//            designed the VM register clearing mechanism to support lifetime management and GC-friendliness.
// 2026-02-17: Introduced the heap and converted string constants into GC-managed string objects;
//            ensured they are correctly processed and reclaimed during the runtime phase.
// 2026-02-18: Major Architectural Evolution:
//            [Dispatch System]: Introduced a decoupled `dispatch` module, centralizing instruction execution logic;
//            implemented a complete suite of logical comparison opcodes (LT, GT, LE, GE) with full support for Number
//            and String (lexicographical) operands; established the PC-skip pattern for conditional branching.
//            [Error Handling & Diagnostics]: Developed a robust Error Handling System with a detailed `VMError` hierarchy;
//            integrated a "Stack Traceback" mechanism to provide deep-dive diagnostics (#0 to #n frame recovery)
//            during runtime panics or type mismatches.
//            [GC & Memory Strategy]: Refined the Mark-and-Sweep algorithm to be type-aware, ensuring explicit
//            destructor (Drop) execution for Heap-allocated Strings and Tables;
//            implemented synchronized string-pool cleanup during the sweep phase to prevent dangling pointers;
//            Optimized performance by deprecating aggressive register auto-nulling in favor of a stable,
//            frame-level reclamation strategy, resolving critical "Nil" value propagation bugs during cross-instruction execution.
// 2026-02-19: Add more debug messages for instruction execution and GC events, providing better visibility into the VM's internal workings during development and testing.
pub mod dispatch;
pub mod error;
pub mod heap;
pub mod stack;
mod std_lib;

use crate::backend::translator::emitter::BytecodeEmitter;
use crate::backend::translator::scanner::{Lifetime, Scanner};
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::heap::Heap;
use crate::backend::vm::stack::{GlobalStack, StackFrame};
use crate::common::object::LuaValue;
use crate::common::object::{GCObject, HeaderOnly, ObjectKind};
use crate::common::opcode::OpCode;
use crate::frontend::ir::{IRGenerator, IRModule, IRUpVal};
use crate::backend::vm::std_lib::{lua_builtin_print};
use std::collections::HashMap;
use std::io::Write;
use std::ops::Index;
use clap::ValueEnum;
use crate::backend::vm::LogLevel::Release;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Release, // 仅输出程序结果
    Debug,   // 输出基本编译信息
    Trace,   // 输出全量寄存器生命周期、IR 和虚拟机指令追踪
}

pub struct FuncMetadata {
    pub bytecode: Vec<OpCode>,
    pub constants: Vec<LuaValue>,
    pub num_locals: usize,
    pub max_stack_size: usize,
    pub reg_metadata: HashMap<usize, Lifetime>,
    pub upvalues_metadata: Vec<IRUpVal>,
    pub child_protos: Vec<String>,
}

const MAX_CALL_STACK: usize = 1000;
const HARD_MEMORY_LIMIT: usize = 1024 * 1024 * 512;//512MB
const VM_THRESHOLD: usize = 1024 * 1024 ;//1MB

pub struct VirtualMachine {
    pub call_stack: Vec<StackFrame>,
    pub value_stack: GlobalStack,
    pub globals: HashMap<String, LuaValue>,
    pub module: IRModule,
    pub func_meta: HashMap<String, FuncMetadata>,
    pub heap: Heap,
    pub log_level: LogLevel,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            value_stack: GlobalStack::default(),
            globals: HashMap::new(),
            module: IRModule { functions: vec![] },
            func_meta: HashMap::new(),
            heap: Heap::new(),
            log_level: Release,
        }
    }

    /// IR 扫描 -> 寄存器分配 -> 字节码生成 -> 入口帧准备
    pub fn init(&mut self, generator: &IRGenerator, log_level: LogLevel, scanner: &mut Scanner) {
        self.log_level = log_level;
        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] VM initialization started with log level: {:?}", self.log_level);
            println!("[DEBUG] Starting scanner..."); std::io::stdout().flush().unwrap();
        }
        self.module = generator.get_module().clone();
        
        for func_ir in &self.module.functions {
            let func_name = &func_ir.name;

            let (num_locals, max_usage) = scanner
                .func_stack_info
                .get(func_name)
                .cloned()
                .unwrap_or((0, 0));

            let mut reg_info_map = HashMap::new();
            for ((f_name, var_kind), &phys_idx) in &scanner.reg_map {
                if f_name == func_name {
                    if let Some(lt) = scanner.lifetimes.get(&(f_name.clone(), var_kind.clone())) {
                        reg_info_map.insert(phys_idx, lt.clone());
                    }
                }
            }

            if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
                println!("[DEBUG] Finished scanning"); std::io::stdout().flush().unwrap();
                println!("[DEBUG] Starting emitter..."); std::io::stdout().flush().unwrap();
            }
            
            let emitter = BytecodeEmitter::new(func_ir, &scanner);
            let (bytecode, constants) = emitter.emit();

            let meta = FuncMetadata {
                bytecode,
                constants,
                num_locals,
                max_stack_size: max_usage + 2,//FIXME:这里的 +2 是为了给函数调用时的返回地址和参数留出空间，后续可以根据实际情况调整
                reg_metadata: reg_info_map,
                upvalues_metadata: func_ir.upvalues.values().cloned().collect(),
                child_protos: func_ir.sub_functions.clone(),
            };

            self.func_meta.insert(func_name.clone(), meta);
        }

        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Finished emit"); std::io::stdout().flush().unwrap();
            println!("[DEBUG] Loading standard library..."); std::io::stdout().flush().unwrap();
        }
        
        self.load_standard_library();

        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Loading finalize constants..."); std::io::stdout().flush().unwrap();
        }
        
        self.finalize_constants();

        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Preparing entry frame..."); std::io::stdout().flush().unwrap();
        }
        
        self.prepare_entry_frame();

        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!(
                "[DEBUG] Initialization successful: {} function metadata resolved. Entry point '_start' initialized (stack_size: {}).",
                self.func_meta.len(),
                self.func_meta
                    .get("_start")
                    .map(|m| m.max_stack_size)
                    .unwrap_or(0)
            )
        }
    }

    pub fn load_standard_library(&mut self) {
        self.globals.insert("print".to_string(), LuaValue::CFunc(lua_builtin_print));
        //TODO:完成其他标准库注册
    }

    // util function to calculate the actual top of the stack for the current frame
    // 0       1           m       m+1    m+2        m+n
    // [value] [value] ... [value] [arg1] [arg2] ... [argN]
    //                             | -> returns m+1
    //
    // this is because, in the new call convention
    // caller can dynamically push args into the stack,
    // however these args actually belong to the callee
    fn get_actual_stack_top(&self) -> usize {
        self.call_stack
            .last()
            .map(|frame| frame.base_offset + frame.reg_count)
            .unwrap_or(0)
    }

    fn make_stack_frame(
        &mut self, 
        func_name: &str, 
        frame_size: usize, 
        return_dest: Option<usize>,
        upvalues: Vec<LuaValue>
    ) -> StackFrame {
        let base_offset = self.get_actual_stack_top();
        self.value_stack.reserve(base_offset + frame_size);
        StackFrame::new(
            func_name.to_string(), 
            return_dest, 
            base_offset,
            frame_size,
            upvalues,
        )
    }

    fn prepare_entry_frame(&mut self) {
        let entry_name = "_start";
        if let Some(meta) = self.func_meta.get(entry_name) {
            let entry_frame = self.make_stack_frame(
                entry_name, 
                meta.max_stack_size,
                None,
                vec![],
            );
            self.call_stack.push(entry_frame);
        } else {
            panic!(
                "[ERROR] SymbolResolutionError: entry point '{}' not found. Ensure the IR generation phase emitted the mandatory entry symbol.",
                entry_name
            );
        }
    }

    pub fn run(&mut self) {
        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Starting execution engine...");
        }

        if self.call_stack.is_empty() {
            panic!("[ERROR] IllegalStateException: call stack is uninitialized. No entry frame found.");
        }
        //loop
        while !self.call_stack.is_empty() {
            // 核心步骤：获取当前栈帧和指令，执行指令，并更新 PC
            let result = self.protected_step();

            if let Err(e) = result {
                self.report_error(e);
                self.call_stack.clear();
                return;
            }

            //GC
            if self.heap.check_gc_condition() {
                self.heap.expand_threshold();
                self.mark_objects();
                self.sweep_objects();
            }
        }
        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Max memory allocated during execution: {} bytes", self.heap.max_allocated);
            
        }
        println!("Program exited with code 0.");
    }
    fn protected_step(&mut self) -> Result<(), VMError> {
        let (func_name, pc) = {
            let frame = self
                .call_stack
                .last()
                .ok_or_else(|| self.error(ErrorKind::InternalError(
                    "IllegalStateException: attempt to step execution on an empty call stack".into()
                )))?;
            (frame.func_name.clone(), frame.pc)
        };

        let meta = self.func_meta.get(&func_name).ok_or_else(|| {
            self.error(ErrorKind::InternalError(format!(
                "ResolutionException: failed to resolve metadata for function symbol '{}'",
                func_name
            )))
        })?;

        if pc >= meta.bytecode.len() {
            return Err(self.error(ErrorKind::InternalError(format!(
                "InstructionOutOfBoundsException: PC ({:04}) exceeded bytecode range for function '{}' (total instructions: {})",
                pc,
                func_name,
                meta.bytecode.len()
            ))));
        }

        let old_stack_depth = self.call_stack.len();

        let curr_instr = meta.bytecode[pc];

        // // --- 新增调试打印开始 ---
        // print!("[TRACE] {:<10} | PC: {:03} | Instr: {:<20} | ", func_name, pc, format!("{:?}", curr_instr));
        // std::io::stdout().flush().unwrap();
        //
        // // 打印当前活跃寄存器状态 (前 8 个通常足够看到循环变量)
        // let frame = self.call_stack.last().unwrap();
        // print!("Regs: ");
        // for i in 0..8 {
        //     if i < frame.registers.len() {
        //         print!("R{}:{:?} ", i, frame.registers[i]);
        //     }
        // }
        // println!();
        // // --- 新增调试打印结束 ---

        self.execute_instruction(curr_instr)?;

        // 不在这里统一将pc加1，而是让每条指令的处理函数根据需要自行调整PC（例如跳转指令会直接修改PC，而普通指令则在执行完后自动加1）

        // 先废弃这个寄存器清理机制
        // self.cleanup_expired_registers();

        Ok(())
    }

    fn report_error(&self, err: VMError) {
        let sep = "=".repeat(70);
        eprintln!("\n{}", sep);

        eprintln!("  {}", err.get_message());

        eprintln!(
            "  Location: Function '{}' at instruction offset [PC: {:04}]",
            err.func_name, err.pc
        );
        eprintln!("{}", sep);

        eprintln!("  Stack Traceback (most recent call first):");
        if err.stack_trace.is_empty() {
            eprintln!("    <empty_stack>");
        } else {
            for (i, frame_name) in err.stack_trace.iter().enumerate().rev() {
                eprintln!("    #{:<2} at {}()", i, frame_name);
            }
        }
        eprintln!("{}\n", sep);
    }

    pub fn error(&self, kind: ErrorKind) -> VMError {
        let (func_name, pc) = if let Some(frame) = self.call_stack.last() {
            (frame.func_name.clone(), frame.pc)
        } else {
            ("<unknown_context>".to_string(), 0)
        };

        let stack_trace = self
            .call_stack
            .iter()
            .map(|f| f.func_name.clone())
            .collect();

        VMError {
            kind,
            func_name,
            pc,
            stack_trace,
        }
    }

    #[allow(dead_code)]
    fn cleanup_expired_registers(&mut self) {
        if let Some(frame) = self.call_stack.last_mut() {
            if let Some(meta) = self.func_meta.get(&frame.func_name) {
                for (&idx, lt) in &meta.reg_metadata {
                    // 修正：只有当 PC 已经走过了生命周期的终点，才设为 Nil
                    // 这样可以确保在 PC == lt.end 的那条指令执行时，数据依然有效
                    if frame.pc > lt.end {
                        // 只有当前不是 Nil 时才操作，减少不必要的赋值
                        if !matches!(frame.get_reg(idx, &self.value_stack), LuaValue::Nil) {
                            frame.set_reg(idx, LuaValue::Nil, &mut self.value_stack);
                        }
                    }
                }
            }
        }
    }

    fn mark_objects(&mut self) {
        unsafe {
            for value in self.globals.values() {
                self.mark_value(value);
            }

            for value in &self.value_stack.values {
                self.mark_value(value);
            }

            for meta in self.func_meta.values() {
                for value in &meta.constants {
                    self.mark_value(value);
                }
            }

            for stack_frame in &self.call_stack {
                // for stack frames, mark upvalues
                for upval in &stack_frame.upvalues {
                    self.mark_value(upval);
                }
            }
        }
    }

    fn sweep_objects(&mut self) {
        unsafe {
            let mut p_prev: *mut GCObject<HeaderOnly> = std::ptr::null_mut();
            let mut p_curr = self.heap.all_objects;

            // use for debug and performance monitoring
            let mut swept_count = 0;
            let mut swept_bytes = 0;

            while !p_curr.is_null() {
                if (*p_curr).mark {
                    (*p_curr).mark = false;
                    p_prev = p_curr;
                    p_curr = (*p_curr).next;
                } else {
                    let p_next = (*p_curr).next;
                    if p_prev.is_null() {
                        self.heap.all_objects = p_next;
                    } else {
                        (*p_prev).next = p_next;
                    }

                    let kind = (*p_curr).kind;
                    let obj_size = (*p_curr).size;

                    swept_count += 1;
                    swept_bytes += obj_size;

                    self.heap.total_allocated = self.heap.total_allocated.saturating_sub(obj_size);

                    match kind {
                        ObjectKind::String => {
                            let str_ptr = p_curr as *mut GCObject<String>;
                            self.heap.string_pool.remove(&(*str_ptr).data);
                            let _ = Box::from_raw(str_ptr);
                        }
                        ObjectKind::Table => {
                            let _ = Box::from_raw(
                                p_curr as *mut GCObject<crate::common::object::LuaTable>,
                            );
                        }
                        ObjectKind::Function => {
                            let _ = Box::from_raw(
                                p_curr as *mut GCObject<crate::common::object::LFunction>,
                            );
                        }
                    }

                    p_curr = p_next;
                }
            }

            //use for debug and performance monitoring
            if swept_count > 0 && matches!(self.log_level, LogLevel::Debug | LogLevel::Trace){
                println!(
                    "[DEBUG] Sweep phase finished: reclaimed {} objects, {} bytes released. Current heap: {} bytes.",
                    swept_count,
                    swept_bytes,
                    self.heap.total_allocated
                );
            }
        }
    }

    unsafe fn mark_value(&self, value: &LuaValue) {
        unsafe {
            match value {
                LuaValue::String(ptr) => {
                    self.mark_raw(*ptr as *mut GCObject<HeaderOnly>);
                }
                LuaValue::Table(ptr) => {
                    if self.mark_raw(*ptr as *mut GCObject<HeaderOnly>) {
                        let table_inner = &(*(*ptr)).data;

                        for (k, v) in &table_inner.data {
                            self.mark_value(k);
                            self.mark_value(v);
                        }

                        if let Some(mt_ptr) = table_inner.metatable {
                            self.mark_value(&LuaValue::Table(mt_ptr));
                        }
                    }
                }
                LuaValue::Function(ptr) => {
                    if self.mark_raw(*ptr as *mut GCObject<HeaderOnly>) {
                        for val in &(*(*ptr)).data.constants {
                            self.mark_value(val);
                        }
                        for upval in &(*(*ptr)).data.upvalues {
                            self.mark_value(upval);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    unsafe fn mark_raw(&self, ptr: *mut GCObject<HeaderOnly>) -> bool {
        if ptr.is_null() || (*ptr).mark {
            return false;
        }
        (*ptr).mark = true;
        true
    }

    pub fn dump_internal_state(&self) {
        let sep = "=".repeat(50);
        println!("\n{}", sep);
        println!("         VIRTUAL MACHINE INTERNAL STATE");
        println!("{}", sep);

        println!("\n[1. Function Metadata & Opcodes]");
        for (name, meta) in &self.func_meta {
            println!("Function: {}", name);
            println!(
                "  Locals: {}, Max Stack: {}",
                meta.num_locals, meta.max_stack_size
            );
            println!("  Constants: {:?}", meta.constants);
            println!("  Bytecode:");
            for (pc, op) in meta.bytecode.iter().enumerate() {
                println!("    [{:03}] {}", pc, op);
            }
            println!("  Register Lifetimes:");
            let mut sorted_regs: Vec<_> = meta.reg_metadata.keys().collect();
            sorted_regs.sort();
            for reg in sorted_regs {
                let lt = &meta.reg_metadata[reg];
                println!("    R{} : start={}, end={}", reg, lt.start, lt.end);
            }
            println!("{}", "-".repeat(30));
        }

        println!("\n[2. Current Call Stack]");
        if self.call_stack.is_empty() {
            println!("  (Stack is empty)");
        } else {
            for (depth, frame) in self.call_stack.iter().enumerate() {
                println!("  Frame #{} -> Function: {}", depth, frame.func_name);
                println!("    PC: {}", frame.pc);
                print!("    Registers: ");
                for i in 0..frame.reg_count {
                    print!("R{}:{:?} ", i, frame.get_reg(i, &self.value_stack));
                }
                println!();
            }
        }

        println!("\n[3. Global Stack]");
        for (idx, val) in self.value_stack.values.iter().enumerate() {
            println!("  [{}] {:?}", idx, val);
        }

        println!("{}\n", "=".repeat(50));
    }

    //用于将所有临时字符串常量转换为 GC 管理的字符串对象，确保在运行时阶段它们能被正确处理和回收
    pub fn finalize_constants(&mut self) {
        for meta in self.func_meta.values_mut() {
            for val in &mut meta.constants {
                if let LuaValue::TempString(_) = val {
                    if let LuaValue::TempString(raw_s) = std::mem::replace(val, LuaValue::Nil) {
                        let gc_ptr = self.heap.alloc_string(raw_s)
                            .expect("BootstrapError: OutOfMemory during constant pool string interning");
                        *val = LuaValue::String(gc_ptr);
                    }
                }
            }
        }
        if matches!(self.log_level, LogLevel::Debug | LogLevel::Trace) {
            println!("[DEBUG] Constant pool resolution completed. Runtime environment is ready.");
        }
    }

    fn get_reg(&self, idx: usize) -> &LuaValue {
        &self.call_stack.last().unwrap().get_reg(idx, &self.value_stack)
    }

    fn set_reg(&mut self, idx: usize, val: LuaValue) {
        self.call_stack.last_mut().unwrap().set_reg(idx, val, &mut self.value_stack);
    }

    fn get_constant(&self, idx: usize) -> &LuaValue {
        let frame = self.call_stack.last().unwrap();
        &self.func_meta.get(&frame.func_name).unwrap().constants[idx]
    }

    fn get_constant_string(&self, idx: usize) -> Result<String, VMError> {
        match self.get_constant(idx) {
            LuaValue::String(ptr) => unsafe { Ok((*(*ptr)).data.clone()) },
            _ => Err(self.error(ErrorKind::InternalError(format!(
                "LinkageError: expected string constant at index {} was not found or has invalid type",
                idx
            )))),
        }
    }
}

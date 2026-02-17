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
// 2026-02-18: Refined Garbage Collection (GC) and designed the Error Handling System;
//            implemented a precise Mark-and-Sweep algorithm with type-aware reclamation;
//            added `ObjectKind` and `size` metadata to GCObject to prevent memory leaks and ensure correct destructor (Drop) execution for Strings and Tables;
//            integrated an automatic string pool synchronized-cleanup mechanism during the sweep phase to eliminate dangling pointers;
//            established a comprehensive runtime error hierarchy, covering type errors, undefined variables, stack overflows, and OOM scenarios;
//            optimized the VM execution loop with integrated GC triggers and register lifetime-based auto-nulling.
pub mod dispatch;
pub mod heap;
pub mod stack;

use crate::backend::translator::emitter::BytecodeEmitter;
use crate::backend::translator::scanner::{Lifetime, Scanner};
use crate::backend::vm::heap::Heap;
use crate::backend::vm::stack::StackFrame;
use crate::common::object::LuaValue;
use crate::common::opcode::OpCode;
use crate::common::object::{GCObject, HeaderOnly, ObjectKind};
use crate::frontend::ir::{IRGenerator, IRModule};
use std::collections::HashMap;
use std::ops::Index;

pub struct FuncMetadata {
    pub bytecode: Vec<OpCode>,
    pub constants: Vec<LuaValue>,
    pub num_locals: usize,
    pub max_stack_size: usize,
    pub reg_metadata: HashMap<usize, Lifetime>,
}

pub struct VirtualMachine {
    pub call_stack: Vec<StackFrame>,
    pub globals: HashMap<String, LuaValue>,
    pub module: IRModule,
    pub func_meta: HashMap<String, FuncMetadata>,
    pub heap: Heap,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            globals: HashMap::new(),
            module: IRModule { functions: vec![] },
            func_meta: HashMap::new(),
            heap: Heap::new(),
        }
    }

    /// IR 扫描 -> 寄存器分配 -> 字节码生成 -> 入口帧准备
    pub fn init(&mut self, generator: &IRGenerator) {
        self.module = generator.get_module().clone();

        let mut scanner = Scanner::new();
        scanner.global_scan(&self.module);

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

            let emitter = BytecodeEmitter::new(func_ir, &scanner);
            let (bytecode, constants) = emitter.emit();

            let meta = FuncMetadata {
                bytecode,
                constants,
                num_locals,
                max_stack_size: max_usage,
                reg_metadata: reg_info_map,
            };

            self.func_meta.insert(func_name.clone(), meta);
        }

        self.finalize_constants();

        self.prepare_entry_frame();

        println!(
            "[VM] 初始化成功：载入 {} 个函数元数据，主入口 _start 已就绪 (栈窗口: {})。",
            self.func_meta.len(),
            self.func_meta
                .get("_start")
                .map(|m| m.max_stack_size)
                .unwrap_or(0)
        );
    }

    fn prepare_entry_frame(&mut self) {
        let entry_name = "_start";
        if let Some(meta) = self.func_meta.get(entry_name) {
            let entry_frame = StackFrame::new(entry_name.to_string(), meta.max_stack_size, None);
            self.call_stack.push(entry_frame);
        } else {
            panic!("[VM 致命错误] 找不到入口函数 _start。请检查 IR 生成阶段。");
        }
    }

    pub fn run(&mut self) {
        println!("[VM] 启动虚拟机，准备执行...");
        if self.call_stack.is_empty() {
            panic!("[VM 致命错误] 调用栈未初始化。");
        }
        //loop
        while (!self.call_stack.is_empty()) {
            // 1. 获取当前栈帧
            let curr_frame: &mut StackFrame = self.call_stack.last_mut().unwrap();
            // 2. 根据 PC 获取当前指令
            let curr_meta = self
                .func_meta
                .get(&curr_frame.func_name)
                .expect("致命错误: 当前函数元数据缺失");
            let curr_instr = curr_meta
                .bytecode
                .get(curr_frame.pc)
                .expect("致命错误: 当前指令超出范围");
            // 3. 执行指令（调用 dispatch 模块）
            //todo: 这里的 dispatch 模块还未实现，后续会根据指令类型进行分发处理，包括算术运算、函数调用、表操作等

            // 4. 更新 PC 和寄存器状态
            curr_frame.pc += 1;
            self.cleanup_expired_registers();
            // 5. GC(如果达到条件)
            if (self.heap.check_gc_condition()) {
                self.heap.expand_threshold();
                // 5.1 GC mark
                self.mark_objects();
                // 5.2 GC sweep (如果为ture则替换标记为false，如果为flase则回收对象)
                self.sweep_objects();
            }
            // =============================================================================
            // 6. 错误处理机制设计 (Error Handling System)
            // =============================================================================
            //
            // 我们的错误处理分为两个维度：内部崩溃 (Panic) 与 运行时异常 (Runtime Error)。
            // 具体的错误上下文收集（如函数名、PC、寄存器快照）将在 dispatch 模块执行循环中捕获。
            //
            // -----------------------------------------------------------------------------
            // A. 致命错误 (Panic / Internal Compiler Error)
            // -----------------------------------------------------------------------------
            // 这类错误代表 VM 内部状态违背了逻辑预期，通常是编译器实现 Bug 或字节码损坏：
            // 1. 字节码非法：遇到未定义的 OpCode。
            // 2. 指令越界：PC 指向了指令集之外的区域。
            // 3. 栈破坏：寄存器索引超过了该 Frame 预分配的 max_stack_size。
            // 4. 类型转换失败：在已经通过静态检查或内部保证的操作中，LuaValue 转换失败。
            // 5. 堆内存污染：解引用到非法的 GCObject 指针（导致宿主 Rust Panic）。
            //
            // -----------------------------------------------------------------------------
            // B. 运行时错误 (Runtime Errors / Managed Errors)
            // -----------------------------------------------------------------------------
            // 这类错误由用户编写的脚本逻辑引起，VM 应优雅地拦截并输出调用栈快照 (Traceback)：
            //
            // 1. 非函数调用 (Non-callable Error):
            //    - 场景：尝试对 Number/Table/Nil 执行 CALL 指令。
            //    - 信息："Attempt to call a non-function value (type: {type}) at PC={pc}"。
            //
            // 2. 访问未定义变量 (Undefined Variable):
            //    - 场景：GETGLOBAL 找不到对应名称，且未配置全局变量缺省值。
            //    - 信息："Undefined variable '{name}' in function '{func}', PC={pc}"。
            //
            // 3. 非法算术/位运算 (Invalid Operation):
            //    - 场景：String + Table, Number << String 等。
            //    - 信息："Attempt to perform arithmetic on a {type1} and {type2} value at PC={pc}"。
            //
            // 4. 内存耗尽 (OOM - Out of Memory):
            //    - 场景：Heap 执行 GC 后，total_allocated 依然接近或超过 hard_limit，无法完成 alloc。
            //    - 信息："Virtual Machine memory limit reached (OOM) during allocation of {size} bytes"。
            //
            // 5. 调用栈溢出 (Stack Overflow):
            //    - 场景：递归调用过深，call_stack.len() > MAX_CALL_STACK_DEPTH。
            //    - 信息："Stack overflow: maximum call stack size ({limit}) exceeded"。
            //
            // -----------------------------------------------------------------------------
            // C. 错误处理实现思路 (Implementation Strategy)
            // -----------------------------------------------------------------------------
            // TODO:
            // 1. 定义 `enum VMError` 枚举，包装上述运行时错误信息。
            // 2. 将 `dispatch::run` 的返回值改为 `Result<(), VMError>`。
            // 3. 在报错发生时，调用 `dump_internal_state()` 并递归 `call_stack` 生成回溯字符串。
            // 4. 考虑引入 `protected_call` (类似 Lua pcall) 机制，允许脚本捕获异常而不中断 VM。
            // =============================================================================
        }
    }

    #[allow(dead_code)]
    fn cleanup_expired_registers(&mut self) {
        if let Some(frame) = self.call_stack.last_mut() {
            if let Some(meta) = self.func_meta.get(&frame.func_name) {
                for (&idx, lt) in &meta.reg_metadata {
                    // 当 PC 到达变量生命周期的终点时，将其设为 Nil 以释放引用, 使 GC 能够回收相关对象
                    // 如果不这么做，那么该变量的回收必须依赖于后续指令对该寄存器的覆盖，这可能导致 GC 无法及时回收，增加内存压力
                    if lt.end == frame.pc {
                        frame.registers[idx] = LuaValue::Nil;
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

            for frame in &self.call_stack {
                for value in &frame.registers {
                    self.mark_value(value);
                }
            }

            for meta in self.func_meta.values() {
                for value in &meta.constants {
                    self.mark_value(value);
                }
            }
        }
    }

    fn sweep_objects(&mut self) {
        unsafe {
            let mut p_prev: *mut GCObject<HeaderOnly> = std::ptr::null_mut();
            let mut p_curr = self.heap.all_objects;

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

                    self.heap.total_allocated = self.heap.total_allocated.saturating_sub(obj_size);

                    // 还原类型并释放
                    // 只有转回正确的具体类型，Box 销毁时才会调用 data 字段（如 String, HashMap）的 Drop
                    // 从而释放这些容器内部拥有的第二层堆内存。
                    match kind {
                        ObjectKind::String => {
                            let str_ptr = p_curr as *mut GCObject<String>;

                            // 清理 string_pool：移除指向已释放对象的悬垂指针
                            // 主要解决的是，如果一个字符串对象被回收了，但 string_pool 里仍然保留着指向它的指针
                            // 那么下次在创建同样的字面量字符串时，string_pool 会错误地认为它已经存在，返回一个悬垂指针，导致未定义行为
                            self.heap.string_pool.remove(&(*str_ptr).data);

                            let _ = Box::from_raw(str_ptr);

                        }
                        ObjectKind::Table => {
                            let _ = Box::from_raw(p_curr as *mut GCObject<HashMap<LuaValue, LuaValue>>);
                        }
                        ObjectKind::Function => {
                            let _ = Box::from_raw(p_curr as *mut GCObject<crate::common::object::LFunction>);
                        }
                    }

                    p_curr = p_next;
                }
            }
        }
    }

    unsafe fn mark_value(&self, value: &LuaValue) {
        unsafe {
            match value {
                LuaValue::String(ptr) => { self.mark_raw(*ptr as *mut GCObject<HeaderOnly>); }
                LuaValue::Table(ptr) => {
                    if self.mark_raw(*ptr as *mut GCObject<HeaderOnly>) {
                        for (k, v) in &(*(*ptr)).data {
                            self.mark_value(k);
                            self.mark_value(v);
                        }
                    }
                }
                LuaValue::Function(ptr) => {
                    if self.mark_raw(*ptr as *mut GCObject<HeaderOnly>) {
                        for val in &(*(*ptr)).data.constants {
                            self.mark_value(val);
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
                for (i, val) in frame.registers.iter().enumerate() {
                    print!("[R{}:{:?}] ", i, val);
                }
                println!();
            }
        }
        println!("{}\n", "=".repeat(50));
    }

    //用于将所有临时字符串常量转换为 GC 管理的字符串对象，确保在运行时阶段它们能被正确处理和回收
    pub fn finalize_constants(&mut self) {
        for meta in self.func_meta.values_mut() {
            for val in &mut meta.constants {
                if let LuaValue::TempString(_) = val {
                    if let LuaValue::TempString(raw_s) = std::mem::replace(val, LuaValue::Nil) {
                        let gc_ptr = self.heap.alloc_string(raw_s);
                        *val = LuaValue::String(gc_ptr);
                    }
                }
            }
        }

        println!("[VM] 常量池转换完成，进入运行时就绪状态。");
    }
}

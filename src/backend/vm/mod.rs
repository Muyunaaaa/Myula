pub mod dispatch;
pub mod heap;
pub mod stack;

use crate::common::object::{LuaValue, LFunction};
use crate::backend::translator::scanner::{Scanner, VarKind, Lifetime};
use crate::frontend::ir::{IRModule, IRGenerator};
use crate::backend::vm::stack::StackFrame;
use crate::backend::translator::emitter::BytecodeEmitter;
use std::collections::HashMap;
use crate::common::opcode::OpCode;

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
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            globals: HashMap::new(),
            module: IRModule { functions: vec![] },
            func_meta: HashMap::new(),
        }
    }

    /// 初始化虚拟机：IR 扫描 -> 寄存器分配 -> 字节码生成 -> 入口帧准备
    pub fn init(&mut self, generator: &IRGenerator) {
        self.module = generator.get_module().clone();

        let mut scanner = Scanner::new();
        scanner.global_scan(&self.module);

        // 3. 为 IR 中的每个函数生成元数据和字节码
        for func_ir in &self.module.functions {
            let func_name = &func_ir.name;

            // 获取 Scanner 分配的栈压力信息 (num_locals, max_stack_size)
            let (num_locals, max_usage) = scanner.func_stack_info.get(func_name)
                .cloned()
                .unwrap_or((0, 0));

            // 构建物理寄存器生命周期映射
            let mut reg_info_map = HashMap::new();
            for ((f_name, var_kind), &phys_idx) in &scanner.reg_map {
                if f_name == func_name {
                    if let Some(lt) = scanner.lifetimes.get(&(f_name.clone(), var_kind.clone())) {
                        reg_info_map.insert(phys_idx, lt.clone());
                    }
                }
            }

            // 核心步骤：调用 Emitter 将该函数的 IR 指令流转换为 OpCode
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

        // 4. 准备主入口 (_start) 的栈帧
        self.prepare_entry_frame();

        println!(
            "[VM] 初始化成功：载入 {} 个函数元数据，主入口 _start 已就绪 (栈窗口: {})。",
            self.func_meta.len(),
            self.func_meta.get("_start").map(|m| m.max_stack_size).unwrap_or(0)
        );
    }

    fn prepare_entry_frame(&mut self) {
        let entry_name = "_start";
        if let Some(meta) = self.func_meta.get(entry_name) {
            // 根据元数据中的 max_stack_size 分配寄存器数组
            let entry_frame = StackFrame::new(
                entry_name.to_string(),
                meta.max_stack_size,
                None, // 入口函数没有返回地址
            );
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

        // 此处逻辑暂不实现，将由未来的 dispatch 模块处理指令循环
    }

    /// 寄存器清理逻辑：根据 Scanner 提供的生命周期，在指令执行后释放不再使用的寄存器
    #[allow(dead_code)]
    fn cleanup_expired_registers(&mut self) {
        // 获取当前顶层栈帧
        if let Some(frame) = self.call_stack.last_mut() {
            if let Some(meta) = self.func_meta.get(&frame.func_name) {
                for (&idx, lt) in &meta.reg_metadata {
                    // 当 PC 到达变量生命周期的终点时，将其设为 Nil 以释放引用（对 GC 友好）
                    if lt.end == frame.pc {
                        frame.registers[idx] = LuaValue::Nil;
                    }
                }
            }
        }
    }

    pub fn dump_internal_state(&self) {
        let sep = "=".repeat(50);
        println!("\n{}", sep);
        println!("         VIRTUAL MACHINE INTERNAL STATE");
        println!("{}", sep);

        println!("\n[1. Function Metadata & Opcodes]");
        for (name, meta) in &self.func_meta {
            println!("Function: {}", name);
            println!("  Locals: {}, Max Stack: {}", meta.num_locals, meta.max_stack_size);
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
}
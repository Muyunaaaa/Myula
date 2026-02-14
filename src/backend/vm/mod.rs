pub mod dispatch;
pub mod heap;
pub mod stack;

use crate::common::object::LuaValue;
use crate::backend::translator::scanner::{Scanner, Lifetime};
use crate::frontend::ir::{IRModule, IRGenerator};
use crate::backend::vm::stack::StackFrame;
use std::collections::HashMap;

pub struct FuncMetadata {
    pub num_locals: usize,
    pub max_usage: usize,
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

    pub fn run(&mut self) {
        println!("[VM] 启动虚拟机，准备执行...");
        if self.call_stack.is_empty() {
            panic!("[VM 致命错误] 调用栈未初始化。请确保 init 阶段正确设置了入口函数的栈帧。");
        }
        //todo:在这里开始执行指令，逐条解释并更新栈帧状态
    }
    pub fn init(&mut self, generator: &IRGenerator) {
        self.module = generator.get_module().clone();

        let mut scanner = Scanner::new();
        scanner.global_scan(&self.module);

        for (func_name, (num_locals, max_usage)) in &scanner.func_stack_info {
            let mut reg_info_map = HashMap::new();

            for ((f_name, var_kind), &phys_idx) in &scanner.reg_map {
                if f_name == func_name {
                    if let Some(lt) = scanner.lifetimes.get(&(f_name.clone(), var_kind.clone())) {
                        reg_info_map.insert(phys_idx, lt.clone());
                    }
                }
            }
            let meta = FuncMetadata {
                num_locals: *num_locals,
                max_usage: *max_usage,
                reg_metadata: reg_info_map,
            };
            self.func_meta.insert(func_name.clone(), meta);
        }

        self.prepare_entry_frame();

        //todo:调用emitter将ir转换为字节码并存储在函数元数据中

        println!("[VM] 初始化成功：载入 {} 个函数元数据，主入口已就绪。", self.func_meta.len());
    }
    fn prepare_entry_frame(&mut self) {
        let entry_name = "_start";
        if let Some(meta) = self.func_meta.get(entry_name) {
            let entry_frame = StackFrame::new(
                entry_name.to_string(),
                meta.max_usage,
                None,
            );
            self.call_stack.push(entry_frame);
        } else {
            panic!("[VM 致命错误] 找不到入口函数 _start。请检查 IR 生成阶段。");
        }
    }

    /// [预留字段] 这里的 cleanup 可以在未来 run 中每一条指令执行后调用
    #[allow(dead_code)]
    fn cleanup_expired_registers(&self, frame: &mut StackFrame) {
        if let Some(meta) = self.func_meta.get(&frame.func_name) {
            for (&idx, lt) in &meta.reg_metadata {
                // 只有当 PC 刚好到达终点时，才将物理寄存器设为 Nil
                // 这样可以精确控制内存占用，不再依赖外部 GC 扫描
                if lt.end == frame.pc {
                    frame.registers[idx] = LuaValue::Nil;
                }
            }
        }
    }
}
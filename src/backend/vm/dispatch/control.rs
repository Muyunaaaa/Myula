use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::stack::StackFrame;
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

impl VirtualMachine {
    /// JUMP
    pub fn handle_jump(&mut self, offset: i32) -> Result<(), VMError> {
        let frame = self.call_stack.last_mut().unwrap();
        // 因为 run 循环末尾会自动 pc += 1，所以这里的跳转逻辑需要微调
        // 如果 offset 为 0，实际上是执行下一条指令
        // 我们通过 (offset - 1) 来抵消循环中的自增，或者直接修改 PC
        let new_pc = (frame.pc as i32 + offset) as usize;
        frame.pc = new_pc;

        // 由于 run 循环中有 curr_frame.pc += 1，
        // 我们在此处设置 PC 后，循环末尾会再次加 1。
        // 为了抵消，我们将设置的值减去 1（假设指令是相对于当前指令计算的）。
        frame.pc = (frame.pc as i32 - 1).max(0) as usize;
        Ok(())
    }

    /// CALL
    pub fn handle_call(&mut self, func_reg: u16, argc: u8, retc: u8) -> Result<(), VMError> {
        let func_val = self.get_reg(func_reg as usize).clone();

        if self.call_stack.len() >= crate::backend::vm::MAX_CALL_STACK {
            return Err(self.error(ErrorKind::StackOverflow));
        }

        match func_val {
            LuaValue::Function(ptr) => {
                let func_obj = unsafe { &(*ptr).data };
                let func_name = &func_obj.name;

                let meta = self.func_meta.get(func_name)
                    .ok_or_else(|| self.error(ErrorKind::InternalError(format!(
                        "InternalExecutionException: metadata for function '{}' could not be resolved",
                        func_name
                    ))))?;

                let mut new_frame = StackFrame {
                    func_name: func_name.clone(),
                    registers: vec![LuaValue::Nil; meta.max_stack_size],
                    pc: 0,
                    ret_dest: Some(func_reg as usize),
                };

                for i in 0..(argc as usize) {
                    let arg_val = self.get_reg(func_reg as usize + 1 + i).clone();
                    if i < new_frame.registers.len() {
                        new_frame.registers[i] = arg_val;
                    }
                }

                self.call_stack.push(new_frame);
                Ok(())
            }

            LuaValue::CFunc(c_func) => {
                let func_idx = func_reg as usize;
                let base_reg = func_idx + 1;

                let num_results = c_func(self, base_reg, argc as usize)?;

                if retc > 0 {
                    let expected = (retc - 1) as usize;
                    for i in num_results..expected {
                        self.set_reg(func_idx + i, LuaValue::Nil);
                    }
                }
                Ok(())
            }

            _ => {
                let msg = match func_val {
                    LuaValue::Nil => "NullPointerException: attempt to invoke a nil value".to_string(),
                    _ => format!(
                        "TypeMismatchException: object of type '{:?}' is not callable",
                        func_val
                    ),
                };
                Err(self.error(ErrorKind::InvalidCall(msg)))
            }
        }
    }
    /// RETURN
    pub fn handle_return(&mut self, start: u16, count: u8) -> Result<(), VMError> {
        let mut results = Vec::new();
        for i in 0..(count as usize) {
            results.push(self.get_reg(start as usize + i).clone());
        }

        let current_frame = self.call_stack.pop()
            .ok_or_else(|| self.error(ErrorKind::InternalError(
                "StackUnderflowException: attempt to return from an empty call stack".into()
            )))?;

        if self.call_stack.is_empty() {
            return Ok(());
        }

        //把 PC 调整回调用指令的位置，确保返回后继续执行下一条指令
        self.call_stack.last_mut().unwrap().pc -= 1;

        if let Some(dest_idx) = current_frame.ret_dest {
            if let Some(caller_frame) = self.call_stack.last_mut() {
                for (i, val) in results.into_iter().enumerate() {
                    let target_idx = dest_idx + i;
                    if target_idx < caller_frame.registers.len() {
                        caller_frame.registers[target_idx] = val;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_halt(&mut self) -> Result<(), VMError> {
        println!("[VM] HALT instruction received. Initiating graceful shutdown sequence...");

        self.call_stack.clear();

        println!("[VM] Execution terminated. Status: Success (0)");

        Ok(())
    }
}
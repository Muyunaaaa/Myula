use crate::backend::vm::VirtualMachine;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::stack::StackFrame;
use crate::common::object::LuaValue;

impl VirtualMachine {
    /// JUMP
    pub fn handle_jump(&mut self, offset: i32) -> Result<(), VMError> {
        let frame = self.call_stack.last_mut().unwrap();
        let new_pc = (frame.pc as i32 + offset) as usize;
        frame.pc = new_pc;
        frame.pc = (frame.pc as i32).max(0) as usize;
        Ok(())
    }

    /// CALL
    pub fn handle_call(&mut self, func_reg: u16, argc: u8, retc: u8) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
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

                let new_frame = self.make_stack_frame(
                    func_name,
                    meta.max_stack_size,
                    Some(func_reg as usize),
                    func_obj.upvalues.clone(),
                );

                self.push_frame(new_frame);
                Ok(())
            }

            LuaValue::CFunc(c_func) => {
                let func_idx = func_reg as usize;

                let stack_top = self.get_actual_stack_top();
                let new_frame = self.make_stack_frame(
                    &format!("__native_{}", func_idx),
                    0,
                    Some(func_idx),
                    vec![],
                );

                // push dummy frame
                self.push_frame(new_frame);
                let num_results = c_func(self, argc as usize)?;

                // restore, clean up dummy frame and args
                self.pop_frame();
                self.value_stack.restore(stack_top);

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
                    LuaValue::Nil => {
                        "NullPointerException: attempt to invoke a nil value".to_string()
                    }
                    _ => format!(
                        "TypeMismatchException: object of type '{:?}' is not callable",
                        func_val
                    ),
                };
                Err(self.error(ErrorKind::InvalidCall(msg)))
            }
        }
    }

    /// PUSH
    pub fn handle_push(&mut self, src: u16) -> Result<(), VMError> {
        let val = self.get_reg(src as usize).clone();
        self.value_stack.push(val);

        self.call_stack.last_mut().unwrap().pc += 1;

        Ok(())
    }

    /// RETURN
    /// 我非常不建议使用这种策略处理返回值，单个还好说，多个又会变得跟之前那种寄存器传参模式一样乱七八糟的
    /// Native Call 那边也是一样
    /// 但如果不做多返回值支持，这个就无所谓了
    /// 行为是对的，但是寄存器乱飞
    /// - Li
    pub fn handle_return(&mut self, start: u16, count: u8) -> Result<(), VMError> {
        // FIXME:目前版本不支持多返回值，后续如果需要支持再改这里
        if count > 1 {
            return Err(self.error(ErrorKind::MultipleReturnValues(
                "MultipleReturnValuesException: returning multiple values is not supported in this VM version".into()
            )));
        }
        let mut results = Vec::new();
        for i in 0..(count as usize) {
            results.push(self.get_reg(start as usize + i).clone());
        }

        let last_frame = self.pop_frame().ok_or_else(|| {
            self.error(ErrorKind::InternalError(
                "StackUnderflowException: attempt to return from an empty call stack".into(),
            ))
        })?;

        if self.call_stack.is_empty() {
            return Ok(());
        }

        if let Some(dest_idx) = last_frame.ret_dest {
            if let Some(caller_frame) = self.call_stack.last_mut() {
                for (i, val) in results.into_iter().enumerate() {
                    let target_idx = dest_idx + i;
                    if target_idx < caller_frame.reg_count {
                        caller_frame.set_reg(target_idx, val, &mut self.value_stack);
                    }
                }
            }
        }

        self.value_stack.restore(last_frame.base_offset);

        Ok(())
    }

    pub fn handle_halt(&mut self) -> Result<(), VMError> {
        println!("[VM] HALT instruction received. Initiating graceful shutdown sequence...");

        self.call_stack.clear();

        println!("[VM] Execution terminated. Status: Success (0)");

        Ok(())
    }
}

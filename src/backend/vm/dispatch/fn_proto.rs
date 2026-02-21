use std::ptr::{null, null_mut};

use crate::backend::vm::VirtualMachine;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::common::object::{GCObject, LuaUpValue, LuaUpValueState, LuaValue};
use crate::frontend::ir::{IRUpVal, IRUpValType};

impl VirtualMachine {
    pub fn handle_fn_proto(&mut self, dest: u16, proto_idx: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let curr_frame = self.call_stack.last().unwrap();

        let curr_meta = self.func_meta.get(&curr_frame.func_name)
            .ok_or_else(|| self.error(ErrorKind::InternalError(
                format!("ResolutionException: failed to resolve metadata for current execution context '{}'", curr_frame.func_name)
            )))?;

        let sub_func_name = curr_meta
            .child_protos
            .get(proto_idx as usize)
            .ok_or_else(|| {
                self.error(ErrorKind::InternalError(format!(
                    "IndexOutOfBoundsException: function prototype index {} is out of range",
                    proto_idx
                )))
            })?;

        let sub_meta = self.func_meta.get(sub_func_name).ok_or_else(|| {
            self.error(ErrorKind::InternalError(format!(
                "LinkageError: symbolic reference to sub-prototype '{}' could not be resolved",
                sub_func_name
            )))
        })?;

        let mut err: Option<ErrorKind> = None;
        let mut out_upvalues: Vec<(usize, *mut GCObject<LuaUpValue>)> = vec![];
        let captured_upvalues: Vec<*mut GCObject<LuaUpValue>> = sub_meta
            .upvalues_metadata
            .iter()
            .map(|upval| match upval.ty {
                IRUpValType::LocalVar(slot) => {
                    let search = curr_frame
                        .out_upvalues
                        .binary_search_by_key(&slot, |(s, _)| *s);
                    if let Ok(idx) = search {
                        // this slot is already captured by current frame, reuse the upvalue object
                        return curr_frame.out_upvalues[idx].1;
                    }

                    let reg_idx = curr_frame.reg_absolute(slot);
                    // create a new open upvalue pointing to the stack slot
                    let upval_ptr = self
                        .heap
                        .alloc_upvalue_object(LuaUpValue {
                            value: LuaUpValueState::Open(reg_idx),
                        })
                        .ok_or_else(|| {
                            err = Some(ErrorKind::OutOfMemory);
                            null_mut::<GCObject<LuaUpValue>>()
                        })
                        .unwrap();
                    out_upvalues.push((slot, upval_ptr));
                    upval_ptr
                }
                IRUpValType::UpVal(slot) => {
                    curr_frame.upvalues.get(slot).unwrap_or(&null_mut()).clone()
                }
            })
            .collect();

        if let Some(e) = err {
            self.error(e);
        }

        // update exported upvalues of current frame
        self.call_stack
            .last_mut()
            .unwrap()
            .out_upvalues
            .append(&mut out_upvalues);

        let new_func = crate::common::object::LFunction {
            name: sub_func_name.clone(),
            opcodes: sub_meta.bytecode.clone(),
            constants: sub_meta.constants.clone(),
            upvalues: captured_upvalues,
            num_locals: sub_meta.num_locals,
            max_stack_size: sub_meta.max_stack_size,
        };

        let func_ptr = self
            .heap
            .alloc_function(new_func)
            .ok_or_else(|| self.error(ErrorKind::OutOfMemory))?;

        self.set_reg(dest as usize, LuaValue::Function(func_ptr));
        Ok(())
    }
}

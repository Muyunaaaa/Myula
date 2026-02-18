use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

impl VirtualMachine {
    pub fn handle_fn_proto(&mut self, dest: u16, proto_idx: u16) -> Result<(), VMError> {
        let curr_frame = self.call_stack.last().unwrap();

        let curr_meta = self.func_meta.get(&curr_frame.func_name)
            .ok_or_else(|| self.error(ErrorKind::InternalError(
                format!("ResolutionException: failed to resolve metadata for current execution context '{}'", curr_frame.func_name)
            )))?;

        let sub_func_name = curr_meta.child_protos.get(proto_idx as usize)
            .ok_or_else(|| self.error(ErrorKind::InternalError(
                format!("IndexOutOfBoundsException: function prototype index {} is out of range", proto_idx)
            )))?;

        let sub_meta = self.func_meta.get(sub_func_name)
            .ok_or_else(|| self.error(ErrorKind::InternalError(
                format!("LinkageError: symbolic reference to sub-prototype '{}' could not be resolved", sub_func_name)
            )))?;

        let new_func = crate::common::object::LFunction {
            name: sub_func_name.clone(),
            opcodes: sub_meta.bytecode.clone(),
            constants: sub_meta.constants.clone(),
            num_locals: sub_meta.num_locals,
            max_stack_size: sub_meta.max_stack_size,
        };

        let func_ptr = self.heap.alloc_function(new_func)
            .ok_or_else(|| self.error(ErrorKind::OutOfMemory))?;

        self.set_reg(dest as usize, LuaValue::Function(func_ptr));
        Ok(())
    }
}
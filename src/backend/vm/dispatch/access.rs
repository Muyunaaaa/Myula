use crate::backend::vm::VirtualMachine;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::common::object::LuaValue;

impl VirtualMachine {
    pub fn handle_move(&mut self, dest: u16, src: u16) -> Result<(), VMError> {
        let val = self.get_reg(src as usize).clone();
        self.set_reg(dest as usize, val);
        self.call_stack.last_mut().unwrap().pc += 1;
        Ok(())
    }

    pub fn handle_loadk(&mut self, dest: u16, const_idx: u16) -> Result<(), VMError> {
        let val = self.get_constant(const_idx as usize).clone();
        self.set_reg(dest as usize, val);
        self.call_stack.last_mut().unwrap().pc += 1;
        Ok(())
    }

    pub fn handle_load_nil(&mut self, dest: u16) -> Result<(), VMError> {
        self.set_reg(dest as usize, LuaValue::Nil);
        self.call_stack.last_mut().unwrap().pc += 1;
        Ok(())
    }

    pub fn handle_load_bool(&mut self, dest: u16, value: bool) -> Result<(), VMError> {
        self.set_reg(dest as usize, LuaValue::Boolean(value));
        self.call_stack.last_mut().unwrap().pc += 1;
        Ok(())
    }

    pub fn handle_get_global(&mut self, dest: u16, name_idx: u16) -> Result<(), VMError> {
        let name = self.get_constant_string(name_idx as usize)?;
        self.call_stack.last_mut().unwrap().pc += 1;
        if let Some(val) = self.globals.get(&name).cloned() {
            self.set_reg(dest as usize, val);
            Ok(())
        } else {
            Err(self.error(ErrorKind::UndefinedVariable(name)))
        }
    }

    pub fn handle_set_global(&mut self, name_idx: u16, src: u16) -> Result<(), VMError> {
        let name = self.get_constant_string(name_idx as usize)?;
        let val = self.get_reg(src as usize).clone();
        self.call_stack.last_mut().unwrap().pc += 1;
        self.globals.insert(name, val);
        Ok(())
    }

    pub fn handle_get_upval(&mut self, dest: u16, upval_idx: u16) -> Result<(), VMError> {
        let curr_frame = self.call_stack.last().unwrap();
        if let Some(upval) = curr_frame.upvalues.get(upval_idx as usize).cloned() {
            self.set_reg(dest as usize, upval);
            self.call_stack.last_mut().unwrap().pc += 1;
            Ok(())
        } else {
            Err(self.error(ErrorKind::UndefinedUpValue(upval_idx)))
        }
    }
}

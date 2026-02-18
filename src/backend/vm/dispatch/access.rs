use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

impl VirtualMachine {
    pub fn handle_move(&mut self, dest: u16, src: u16) -> Result<(), VMError> {
        let val = self.get_reg(src as usize).clone();
        self.set_reg(dest as usize, val);
        Ok(())
    }

    pub fn handle_loadk(&mut self, dest: u16, const_idx: u16) -> Result<(), VMError> {
        let val = self.get_constant(const_idx as usize).clone();
        self.set_reg(dest as usize, val);
        Ok(())
    }

    pub fn handle_load_nil(&mut self, dest: u16) -> Result<(), VMError> {
        self.set_reg(dest as usize, LuaValue::Nil);
        Ok(())
    }

    pub fn handle_load_bool(&mut self, dest: u16, value: bool) -> Result<(), VMError> {
        self.set_reg(dest as usize, LuaValue::Boolean(value));
        Ok(())
    }

    pub fn handle_get_global(&mut self, dest: u16, name_idx: u16) -> Result<(), VMError> {
        let name = self.get_constant_string(name_idx as usize)?;

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

        self.globals.insert(name, val);
        Ok(())
    }
}
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

impl VirtualMachine {
    pub fn handle_compare<F>(&mut self, left: u16, right: u16, op: F) -> Result<(), VMError>
    where F: Fn(&LuaValue, &LuaValue) -> bool
    {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        if !op(v1, v2) {
            let frame = self.call_stack.last_mut().unwrap();
            frame.pc += 1;//通常比较指令后面会跟一个条件跳转指令，如果比较结果不满足条件，就跳过下一条指令
        }
        Ok(())
    }

    /// EQ: R[left] == R[right]
    pub fn handle_eq(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        self.handle_compare(left, right, |a, b| a == b)
    }

    /// NE: R[left] != R[right]
    pub fn handle_ne(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        self.handle_compare(left, right, |a, b| a != b)
    }

    /// LT: R[left] < R[right]
    pub fn handle_lt(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                if !(n1 < n2) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                if !((*(*s1)).data < (*(*s2)).data) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            _ => Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '<' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        }
    }

    /// GT: R[left] > R[right]
    pub fn handle_gt(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                if !(n1 > n2) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                if !((*(*s1)).data > (*(*s2)).data) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            _ => Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '>' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        }
    }

    /// LE: R[left] <= R[right]
    pub fn handle_le(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                if !(n1 <= n2) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                if !((*(*s1)).data <= (*(*s2)).data) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            _ => Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '<=' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        }
    }

    /// GE: R[left] >= R[right]
    pub fn handle_ge(&mut self, left: u16, right: u16) -> Result<(), VMError> {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                if !(n1 >= n2) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                if !((*(*s1)).data >= (*(*s2)).data) { self.call_stack.last_mut().unwrap().pc += 1; }
                Ok(())
            }
            _ => Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '>=' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        }
    }

    /// TEST: 检查 R[reg] 是否为“真”
    /// 如果为“假”(Nil 或 False)，则跳过下一条指令
    pub fn handle_test(&mut self, reg: u16) -> Result<(), VMError> {
        let val = self.get_reg(reg as usize);

        if !val.is_truthy() {
            let frame = self.call_stack.last_mut().unwrap();
            frame.pc += 1;
        }
        Ok(())
    }
}
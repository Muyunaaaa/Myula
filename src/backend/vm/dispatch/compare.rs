use crate::backend::vm::VirtualMachine;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::common::object::LuaValue;

impl VirtualMachine {
    pub fn handle_compare<F>(
        &mut self,
        dest: u16,
        left: u16,
        right: u16,
        op: F,
    ) -> Result<(), VMError>
    where
        F: Fn(&LuaValue, &LuaValue) -> bool,
    {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        let res = op(v1, v2);

        self.set_reg(dest as usize, LuaValue::Boolean(res));

        Ok(())
    }

    /// EQ: R[dest] = (R[left] == R[right])
    pub fn handle_eq(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        self.handle_compare(dest, left, right, |a, b| a == b)
    }

    /// NE: R[dest] = (R[left] != R[right])
    pub fn handle_ne(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        self.handle_compare(dest, left, right, |a, b| a != b)
    }

    /// LT: R[dest] = (R[left] < R[right])
    pub fn handle_lt(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        let res = match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => n1 < n2,
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                (*(*s1)).data < (*(*s2)).data
            },
            _ => return Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '<' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        };

        self.set_reg(dest as usize, LuaValue::Boolean(res));
        Ok(())
    }

    /// GT: R[dest] = (R[left] > R[right])
    pub fn handle_gt(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        let res = match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => n1 > n2,
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                (*(*s1)).data > (*(*s2)).data
            },
            _ => return Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '>' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        };

        self.set_reg(dest as usize, LuaValue::Boolean(res));
        Ok(())
    }

    /// LE: R[dest] = (R[left] <= R[right])
    pub fn handle_le(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        let res = match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => n1 <= n2,
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                (*(*s1)).data <= (*(*s2)).data
            },
            _ => return Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '<=' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        };

        self.set_reg(dest as usize, LuaValue::Boolean(res));
        Ok(())
    }

    /// GE: R[dest] = (R[left] >= R[right])
    pub fn handle_ge(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        let res = match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => n1 >= n2,
            (LuaValue::String(s1), LuaValue::String(s2)) => unsafe {
                (*(*s1)).data >= (*(*s2)).data
            },
            _ => return Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: relational operator '>=' is not defined between '{:?}' and '{:?}'",
                v1, v2
            )))),
        };

        self.set_reg(dest as usize, LuaValue::Boolean(res));
        Ok(())
    }

    /// TEST: 检查 R[reg] 是否为“真”
    /// 如果为“假”(Nil 或 False)，则跳过下一条指令
    pub fn handle_test(&mut self, reg: u16) -> Result<(), VMError> {
        let val = self.get_reg(reg as usize);

        if !val.is_truthy() {
            let frame = self.call_stack.last_mut().unwrap();
            frame.pc += 2;
        } else {
            let frame = self.call_stack.last_mut().unwrap();
            frame.pc += 1;
        }
        Ok(())
    }
}

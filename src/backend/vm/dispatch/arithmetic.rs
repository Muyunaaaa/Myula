use crate::backend::vm::VirtualMachine;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::common::object::LuaValue;
use crate::common::opcode::UnaryOpType;

impl VirtualMachine {
    /// ADD: R[dest] = R[left] + R[right]
    pub fn handle_add(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        self.handle_binary_op(dest, left, right, |n1, n2| n1 + n2, "addition")
    }

    /// SUB: R[dest] = R[left] - R[right]
    pub fn handle_sub(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        self.handle_binary_op(dest, left, right, |n1, n2| n1 - n2, "subtraction")
    }

    /// MUL: R[dest] = R[left] * R[right]
    pub fn handle_mul(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        self.handle_binary_op(dest, left, right, |n1, n2| n1 * n2, "multiplication")
    }

    /// DIV: R[dest] = R[left] / R[right]
    pub fn handle_div(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v2 = self.get_reg(right as usize);
        if let LuaValue::Number(n2) = v2 {
            if *n2 == 0.0 {
                return Err(self.error(ErrorKind::ArithmeticError(
                    "ArithmeticException: division by zero".into(),
                )));
            }
        }
        self.handle_binary_op(dest, left, right, |n1, n2| n1 / n2, "division")
    }

    /// MOD: R[dest] = R[left] % R[right]
    pub fn handle_mod(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v2 = self.get_reg(right as usize);
        if let LuaValue::Number(n2) = v2 {
            if *n2 == 0.0 {
                return Err(self.error(ErrorKind::ArithmeticError(
                    "ArithmeticException: modulo by zero".into(),
                )));
            }
        }
        self.handle_binary_op(dest, left, right, |n1, n2| n1 % n2, "modulo")
    }

    /// UNOP
    pub fn handle_unary_op(&mut self, dest: u16, src: u16, op: UnaryOpType) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let val = self.get_reg(src as usize).clone();

        let res = match op {
            UnaryOpType::Neg => {
                if let LuaValue::Number(n) = val {
                    LuaValue::Number(-n)
                } else {
                    return Err(self.error(ErrorKind::TypeError(format!(
                        "TypeMismatchException: operator '-' is not defined for type '{:?}'",
                        val
                    ))));
                }
            }
            UnaryOpType::Not => LuaValue::Boolean(!val.is_truthy()),
            UnaryOpType::Len => match val {
                LuaValue::String(ptr) => unsafe {
                    let s = &(*ptr).data;
                    LuaValue::Number(s.len() as f64)
                },

                LuaValue::Table(ptr) => unsafe {
                    let table_data = &(*ptr).data.data;
                    LuaValue::Number(table_data.len() as f64)
                },
                _ => {
                    return Err(self.error(ErrorKind::TypeError(format!(
                        "TypeMismatchException: operation '#' (len) is not defined for type '{:?}'",
                        val
                    ))));
                }
            },
        };

        self.set_reg(dest as usize, res);
        Ok(())
    }
    fn handle_binary_op<F>(
        &mut self,
        dest: u16,
        left: u16,
        right: u16,
        op_fn: F,
        op_name: &str,
    ) -> Result<(), VMError>
    where
        F: Fn(f64, f64) -> f64,
    {
        let v1 = self.get_reg(left as usize);
        let v2 = self.get_reg(right as usize);

        match (v1, v2) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                let res = op_fn(*n1, *n2);
                self.set_reg(dest as usize, LuaValue::Number(res));
                Ok(())
            }
            //TODO: 后续支持Table和String的加法等
            _ => {
                let msg = format!(
                    "TypeMismatchException: binary operator '{}' is not defined for types '{:?}' and '{:?}'",
                    op_name, v1, v2
                );
                Err(self.error(ErrorKind::TypeError(msg)))
            }
        }
    }

    /// AND: R[dest] = R[left] and R[right]
    pub fn handle_and(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize).clone();
        let res = if !v1.is_truthy() {
            v1
        } else {
            self.get_reg(right as usize).clone()
        };
        self.set_reg(dest as usize, res);
        Ok(())
    }

    /// OR: R[dest] = R[left] or R[right]
    pub fn handle_or(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize).clone();
        let res = if v1.is_truthy() {
            v1
        } else {
            self.get_reg(right as usize).clone()
        };
        self.set_reg(dest as usize, res);
        Ok(())
    }

    pub fn handle_concat(&mut self, dest: u16, left: u16, right: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let v1 = self.get_reg(left as usize).clone();
        let v2 = self.get_reg(right as usize).clone();

        let s1 = self.value_to_string(&v1)?;
        let s2 = self.value_to_string(&v2)?;

        let combined = s1 + &s2;

        //heap.alloc_string
        let new_str_ptr = self
            .heap
            .alloc_string(combined)
            .ok_or_else(|| self.error(ErrorKind::OutOfMemory))?;

        self.set_reg(dest as usize, LuaValue::String(new_str_ptr));

        Ok(())
    }

    fn value_to_string(&self, val: &LuaValue) -> Result<String, VMError> {
        match val {
            LuaValue::String(ptr) => {
                unsafe { Ok((*(*ptr)).data.clone()) }
            }
            LuaValue::Number(n) => {
                Ok(n.to_string())
            }
            LuaValue::Nil => {
                Err(self.error(ErrorKind::TypeError(
                    "NullPointerException: illegal concatenation of a nil value".into()
                )))
            }
            LuaValue::Boolean(b) => {
                Err(self.error(ErrorKind::TypeError(format!(
                    "TypeMismatchException: boolean type ({}) does not support implicit string conversion for concatenation",
                    b
                ))))
            }
            _ => {
                Err(self.error(ErrorKind::TypeError(format!(
                    "IncompatibleTypesException: cannot perform string concatenation on type '{:?}'",
                    val
                ))))
            }
        }
    }
}

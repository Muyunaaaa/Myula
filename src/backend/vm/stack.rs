// Myula compiler stack frame definitions
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
use crate::common::object::LuaValue;
pub struct StackFrame {
    pub func_name: String,
    pub registers: Vec<LuaValue>,
    pub pc: usize,
    pub ret_dest: Option<usize>,
}

impl StackFrame {
    pub fn new(name: String, size: usize, ret_dest: Option<usize>) -> Self {
        Self {
            func_name: name,
            registers: vec![LuaValue::Nil; size],
            pc: 0,
            ret_dest,
        }
    }
}

impl StackFrame {
    #[inline(always)]
    pub fn get_reg(&self, idx: usize) -> &LuaValue {
        &self.registers[idx]
    }

    #[inline(always)]
    pub fn set_reg(&mut self, idx: usize, val: LuaValue) {
        self.registers[idx] = val;
    }
}
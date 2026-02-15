/*
   函数栈帧实现
*/
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
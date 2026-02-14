use std::collections::HashMap;
use std::fmt;

pub type CFunction = fn(); // 保持你的原始定义

#[repr(C)]
pub struct HeaderOnly;

#[repr(C)]
pub struct GCObject<T> {
    pub mark: bool,
    pub next: *mut GCObject<HeaderOnly>,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LuaValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(*mut GCObject<String>),
    Table(*mut GCObject<HashMap<LuaValue, LuaValue>>),
    Function(*mut GCObject<LFunction>),
    CFunc(CFunction),
    UserData(*mut std::ffi::c_void),
}

#[derive(Debug)]
pub struct LFunction {
    pub opcodes: Vec<crate::common::opcode::OpCode>,
    pub constants: Vec<LuaValue>,
    pub num_locals: usize,    // 局部变量槽位数
    pub max_stack_size: usize, // 窗口大小
}
#[derive(Debug, Clone, Copy)]
pub struct LuaObject {
    pub value: LuaValue,
}

impl LuaObject {
    pub fn new(value: LuaValue) -> Self {
        Self { value }
    }
}
#[derive(Debug, Clone)]
pub struct LuaSymbol {
    pub name: *mut GCObject<String>,
    pub value: LuaValue,
}
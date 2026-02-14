use std::collections::HashMap;
use std::fmt;

pub type CFunction = fn();
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
    Function(*mut GCObject<Vec<u8>>),
    CFunc(CFunction),
    UserData(*mut std::ffi::c_void),
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

impl fmt::Display for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Number(n) => write!(f, "{}", n),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::String(ptr) => unsafe {
                write!(f, "{}", (*(*ptr)).data)
            },
            LuaValue::Table(ptr) => write!(f, "table: {:p}", ptr),
            LuaValue::Function(ptr) => write!(f, "function: {:p}", ptr),
            LuaValue::CFunc(_) => write!(f, "cfunction"),
            LuaValue::UserData(ptr) => write!(f, "userdata: {:p}", ptr),
        }
    }
}
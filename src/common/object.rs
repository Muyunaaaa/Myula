use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

pub type CFunction = fn();

#[repr(C)]
pub struct HeaderOnly;

#[repr(C)]
pub struct GCObject<T> {
    pub mark: bool,
    pub next: *mut GCObject<HeaderOnly>,
    pub data: T,
}

#[derive(Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(*mut GCObject<String>),
    Table(*mut GCObject<HashMap<LuaValue, LuaValue>>),
    Function(*mut GCObject<LFunction>),
    CFunc(CFunction),
    UserData(*mut std::ffi::c_void),
    TempString(String)
}

impl Eq for LuaValue {}

impl Hash for LuaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            LuaValue::Nil => (),
            LuaValue::Number(n) => n.to_bits().hash(state),
            LuaValue::Boolean(b) => b.hash(state),
            LuaValue::String(p) => (*p as usize).hash(state),
            LuaValue::Table(p) => (*p as usize).hash(state),
            LuaValue::Function(p) => (*p as usize).hash(state),
            LuaValue::CFunc(f) => (*f as usize).hash(state),
            LuaValue::UserData(p) => (*p as usize).hash(state),
            LuaValue::TempString(s) => s.hash(state),
        }
    }
}

impl fmt::Debug for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "Nil"),
            LuaValue::Number(n) => write!(f, "Number({})", n),
            LuaValue::Boolean(b) => write!(f, "Bool({})", b),
            LuaValue::String(ptr) => unsafe {
                if ptr.is_null() { write!(f, "String(NULL)") }
                else { write!(f, "String(\"{}\")", (*(*ptr)).data) }
            },
            LuaValue::Table(ptr) => write!(f, "Table({:p})", ptr),
            LuaValue::Function(ptr) => write!(f, "LFunc({:p})", ptr),
            LuaValue::CFunc(_) => write!(f, "CFunc"),
            LuaValue::UserData(ptr) => write!(f, "UserData({:p})", ptr),
            LuaValue::TempString(s) => write!(f, "TempString(\"{}\")", s),
        }
    }
}
impl fmt::Display for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Number(n) => write!(f, "{}", n),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::String(ptr) => unsafe {
                if ptr.is_null() { write!(f, "null") }
                else { write!(f, "\"{}\"", (*(*ptr)).data) }
            },
            LuaValue::TempString(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct LFunction {
    pub opcodes: Vec<crate::common::opcode::OpCode>,
    pub constants: Vec<LuaValue>,
    pub num_locals: usize,
    pub max_stack_size: usize,
}

#[derive(Debug, Clone)]
pub struct LuaObject {
    pub value: LuaValue,
}

#[derive(Clone)]
pub struct LuaSymbol {
    pub name: *mut GCObject<String>,
    pub value: LuaValue,
}

impl fmt::Debug for LuaSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let name_str = if self.name.is_null() { "NULL" } else { &(*self.name).data };
            f.debug_struct("LuaSymbol")
                .field("name", &name_str)
                .field("value", &self.value)
                .finish()
        }
    }
}
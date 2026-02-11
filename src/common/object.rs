/*
   这里主要定义Object
*/
use crate::common::hash;
use std::cell::RefCell;
use std::rc::Rc;
pub type LuaHashRef = Rc<RefCell<hash::LuaHash>>;
pub type CFunction = fn();

// lua-value
#[derive(Debug, Clone)]
pub enum LuaValue {
    MARK,
    NIL,
    NUMBER(f64),
    STRING(Rc<String>),
    BOOLEAN(bool),
    ARRAY(LuaHashRef),
    FUNCTION(Rc<Vec<u8>>),
    CFUNCTION(CFunction),
    USERDATA(*mut std::ffi::c_void),
}

//lua-symbol
#[derive(Debug, Clone)]
struct LuaSymbol {
    name: Rc<String>,
    value: LuaValue,
}

//语法糖

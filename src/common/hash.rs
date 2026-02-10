use crate::common::object::LuaValue;
#[derive(Debug, Clone)]
pub struct LuaNode{
    key: LuaValue,
    value: LuaValue,
    next: Option<Box<LuaNode>>,
}

#[derive(Debug)]
pub struct LuaHash{
    mark: bool,
    nhash: usize,
    list: Vec<Option<Box<LuaNode>>>,
}

impl LuaHash{
}
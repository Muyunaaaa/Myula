// Myula compiler stack frame definitions
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
//      26-02-20: Added GlobalStack struct to manage the global value stack,
//                and updated StackFrame to use base offsets into the global stack
//                instead of maintaining its own local register array
//      26-02-20: Added upvalues field to StackFrame to support closure captures
use crate::common::object::{GCObject, LuaUpValue, LuaValue};

pub struct StackFrame {
    pub func_name: String,
    pub base_offset: usize, // base offset in the global stack for this frame
    pub reg_count: usize,   // number of registers used by this frame
    pub pc: usize,
    pub ret_dest: Option<usize>,
    // upvalues **CAPUTURED** by the function prototype that this frame is executing
    pub upvalues: Vec<*mut GCObject<LuaUpValue>>,
    // upvalues **ESCAPED** from this frame that need to be closed when this frame is popped
    pub out_upvalues: Vec<(usize, *mut GCObject<LuaUpValue>)>,
}

#[derive(Default)]
pub struct GlobalStack {
    pub values: Vec<LuaValue>,
}

impl GlobalStack {
    // reserve space for additional values
    pub fn reserve(&mut self, min_size: usize) {
        let current_len = self.values.len();
        if current_len < min_size {
            self.values.resize(min_size, LuaValue::Nil);
        }
    }

    // push a value onto the stack
    pub fn push(&mut self, val: LuaValue) {
        self.values.push(val);
    }

    // discard values above the given offset
    // used when returning from a function to clean up the stack
    pub fn restore(&mut self, offset: usize) {
        self.values.truncate(offset);
    }
}

impl StackFrame {
    pub fn new(
        name: String,
        ret_dest: Option<usize>,
        base_offset: usize,
        reg_count: usize,
        upvalues: Vec<*mut GCObject<LuaUpValue>>,
    ) -> Self {
        Self {
            func_name: name,
            base_offset,
            pc: 0,
            ret_dest,
            reg_count,
            upvalues,
            out_upvalues: vec![],
        }
    }

    pub fn reg_absolute(&self, idx: usize) -> usize {
        self.base_offset + idx
    }
}

impl<'a> StackFrame {
    #[inline(always)]
    pub fn get_reg(&self, idx: usize, global_stack: &'a GlobalStack) -> &'a LuaValue {
        &global_stack.values[self.base_offset + idx]
    }

    #[inline(always)]
    pub fn set_reg(&mut self, idx: usize, val: LuaValue, global_stack: &mut GlobalStack) {
        global_stack.values[self.base_offset + idx] = val;
    }
}

// Myula compiler heap
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-2-17: Initial implementation of Heap with string interning and basic GC object management; added alloc_string method to manage string objects and maintain a string pool for efficient memory usage and quick lookups.
use crate::common::object::{GCObject, LuaValue, HeaderOnly};
use std::collections::HashMap;

pub struct Heap {
    pub all_objects: *mut GCObject<HeaderOnly>,
    pub string_pool: HashMap<String, *mut GCObject<String>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            all_objects: std::ptr::null_mut(),
            string_pool: HashMap::new(),
        }
    }

    pub fn alloc_string(&mut self, s: String) -> *mut GCObject<String> {
        if let Some(&ptr) = self.string_pool.get(&s) {
            return ptr;
        }
        
        let obj = GCObject {
            mark: false,
            next: self.all_objects, 
            data: s.clone(),
        };
        
        let boxed = Box::new(obj);
        let ptr = Box::into_raw(boxed);
        
        self.all_objects = ptr as *mut GCObject<HeaderOnly>;
        
        self.string_pool.insert(s, ptr);

        ptr
    }
}
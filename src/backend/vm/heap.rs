// Myula compiler heap
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-02-17: Initial implementation of Heap with string interning and basic GC object management;
//            added alloc_string method to manage string objects and maintain a string pool for efficient
//            memory usage and quick lookups.
// 2026-02-18: Major Memory Management & Type-Aware Evolution:
//            [Polymorphic Allocation]: Introduced specialized allocators `alloc_table` and `alloc_function`,
//            supporting the instantiation of complex heap objects beyond raw strings;
//            [Precise Memory Tracking]: Implemented a sophisticated memory accounting system that calculates
//            not just struct sizes, but the heap-allocated capacity of internal containers (String data,
//            HashMap buckets, and OpCode/Constant vectors);
//            [GC Control Logic]: Integrated `check_gc_condition` and `expand_threshold` to implement a
//            dynamic GC triggering mechanism, providing a balance between memory footprint and execution throughput;
//            [Memory Safety]: Added a hard memory limit check (HARD_MEMORY_LIMIT) within `alloc_raw_object`
//            to provide an ultimate safeguard against OOM scenarios in the VM runtime.
// 2026-02-19: Add more debug information for GC tuning, including max_allocated to track peak memory usage during execution, 
//            aiding in optimizing GC thresholds and understanding memory patterns of Lua programs running on the VM.
use crate::common::object::{GCObject, HeaderOnly, ObjectKind, LFunction, LuaValue};
use std::collections::HashMap;

pub struct Heap {
    pub all_objects: *mut GCObject<HeaderOnly>,
    pub string_pool: HashMap<String, *mut GCObject<String>>,
    pub total_allocated: usize,
    pub threshold: usize,
    // used for debugging and tuning GC parameters, not used in actual GC logic
    pub max_allocated: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            all_objects: std::ptr::null_mut(),
            string_pool: HashMap::new(),
            total_allocated: 0,
            threshold: crate::backend::vm::VM_THRESHOLD,
            max_allocated: 0,
        }
    }

    pub fn alloc_string(&mut self, s: String) -> Option<*mut GCObject<String>> {
        if let Some(&ptr) = self.string_pool.get(&s) {
            return Some(ptr);
        }

        let extra_mem = s.capacity();
        let total_size = std::mem::size_of::<GCObject<String>>() + extra_mem;

        if let Some(ptr) = self.alloc_raw_object(s.clone(), ObjectKind::String, total_size) {
            self.string_pool.insert(s, ptr);
            Some(ptr)
        } else {
            None
        }
    }
    pub fn alloc_table(&mut self, table_data: crate::common::object::LuaTable) -> Option<*mut GCObject<crate::common::object::LuaTable>> {
        let size = std::mem::size_of::<GCObject<crate::common::object::LuaTable>>()
            + table_data.data.capacity() * std::mem::size_of::<(LuaValue, LuaValue)>();

        self.alloc_raw_object(table_data, ObjectKind::Table, size)
    }

    pub fn alloc_function(&mut self, data: LFunction) -> Option<*mut GCObject<LFunction>> {
        let size = std::mem::size_of::<GCObject<LFunction>>()
            + data.opcodes.capacity() * std::mem::size_of::<crate::common::opcode::OpCode>()
            + data.constants.capacity() * std::mem::size_of::<LuaValue>();

        self.alloc_raw_object(data, ObjectKind::Function, size)
    }

    fn alloc_raw_object<T>(&mut self, data: T, kind: ObjectKind, size: usize) -> Option<*mut GCObject<T>> {
        if self.total_allocated + size > crate::backend::vm::HARD_MEMORY_LIMIT {
            return None;
        }

        let obj = GCObject {
            mark: false,
            kind,
            size,
            next: self.all_objects,
            data,
        };
        let boxed = Box::new(obj);
        let ptr = Box::into_raw(boxed);
        self.all_objects = ptr as *mut GCObject<HeaderOnly>;

        self.total_allocated += size;

        if(self.total_allocated > self.max_allocated) {
            self.max_allocated = self.total_allocated;
        }

        Some(ptr)
    }

    pub fn check_gc_condition(&mut self) -> bool{
        if self.total_allocated > self.threshold {
            return true;
        }
        return false;
    }

    pub fn expand_threshold(&mut self) {
        self.threshold *= 2;
    }
}
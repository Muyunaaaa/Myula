// Myula compiler heap
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-2-17: Initial implementation of Heap with string interning and basic GC object management; added alloc_string method to manage string objects and maintain a string pool for efficient memory usage and quick lookups.
// 2026-02-18: Refactored Heap with precise memory tracking and type-aware allocation;
//            introduced specialized allocators: `alloc_table` and `alloc_function` to handle complex heap objects;
//            added `ObjectKind` and `size` fields to GCObject to support polymorphic deallocation during GC;
//            implemented accurate memory usage estimation by including capacities of internal containers (String, HashMap, Vec);
//            added GC trigger logic (`check_gc_condition`) and dynamic threshold scaling to balance performance and memory pressure.
use crate::common::object::{GCObject, HeaderOnly, ObjectKind, LFunction, LuaValue};
use std::collections::HashMap;

pub struct Heap {
    pub all_objects: *mut GCObject<HeaderOnly>,
    pub string_pool: HashMap<String, *mut GCObject<String>>,
    pub total_allocated: usize,
    pub threshold: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            all_objects: std::ptr::null_mut(),
            string_pool: HashMap::new(),
            total_allocated: 0,
            threshold: 1024 * 1024,
        }
    }

    pub fn alloc_string(&mut self, s: String) -> *mut GCObject<String> {
        if let Some(&ptr) = self.string_pool.get(&s) {
            return ptr;
        }

        let extra_mem = s.capacity(); // 字符串内部缓冲区的内存
        let total_size = std::mem::size_of::<GCObject<String>>() + extra_mem;

        let obj = GCObject {
            mark: false,
            kind: ObjectKind::String,
            size: total_size,
            next: self.all_objects,
            data: s.clone(),
        };

        let boxed = Box::new(obj);
        let ptr = Box::into_raw(boxed);

        self.all_objects = ptr as *mut GCObject<HeaderOnly>;

        self.string_pool.insert(s, ptr);

        ptr
    }

    pub fn alloc_table(&mut self, data: HashMap<LuaValue, LuaValue>) -> *mut GCObject<HashMap<LuaValue, LuaValue>> {
        // size = GCObject 结构体大小 + HashMap 桶占用的粗略内存
        // HashMap 的内部节点动态分配较难精确统计，此处以 capacity 估算
        let size = std::mem::size_of::<GCObject<HashMap<LuaValue, LuaValue>>>()
            + data.capacity() * std::mem::size_of::<(LuaValue, LuaValue)>();

        let ptr = self.alloc_raw_object(data, ObjectKind::Table, size);
        self.total_allocated += size;
        ptr
    }

    pub fn alloc_function(&mut self, data: LFunction) -> *mut GCObject<LFunction> {
        // size = 结构体大小 + 指令集和常量池的堆内存
        let size = std::mem::size_of::<GCObject<LFunction>>()
            + data.opcodes.capacity() * std::mem::size_of::<crate::common::opcode::OpCode>()
            + data.constants.capacity() * std::mem::size_of::<LuaValue>();

        let ptr = self.alloc_raw_object(data, ObjectKind::Function, size);
        self.total_allocated += size;
        ptr
    }

    fn alloc_raw_object<T>(&mut self, data: T, kind: ObjectKind, size: usize) -> *mut GCObject<T> {
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
        ptr
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
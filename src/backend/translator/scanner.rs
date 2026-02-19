// Myula compiler IR scanner
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-2-14: Implemented a comprehensive IR Scanner providing linear-scan register allocation and lifetime analysis;
//            introduced VarKind to distinguish between temporary registers and local slots while tracking their live ranges via instr_count;
//            added basic type inference for immediate loads and local storage;
//            implemented global variable discovery and function-level stack pressure mapping (func_stack_info) to support downstream code generation and memory management.
// 2026-02-19: Introduced Conservative Stride-based Register Allocation;
//            set a stride of 4 for TEMP registers to force physical isolation and create implicit parameter buffers;
//            fixed Register Aliasing conflicts occurring during parameter passing in CALL instructions,
//            significantly improving compiler and VM stability.
//            Fix the issue of registers not be allocated when reach the FnProto and other instructions

use std::collections::{HashMap, HashSet};
use crate::frontend::ir::{self, IRModule, IRInstruction, IRTerminator, IROperand};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarKind {
    Reg(usize),  // %n
    Slot(usize), // %local_n
}

#[derive(Debug, Clone)]
pub struct Lifetime {
    pub start: usize,
    pub end: usize,
    pub is_fixed: bool,
    pub inferred_type: Option<String>,
}

pub struct Scanner {
    pub lifetimes: HashMap<(String, VarKind), Lifetime>,
    pub global_vars: HashSet<String>,
    pub reg_map: HashMap<(String, VarKind), usize>,
    pub func_stack_info: HashMap<String, (usize, usize)>,
    pub child_protos: HashMap<String, Vec<String>>,
    instr_count: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            lifetimes: HashMap::new(),
            global_vars: HashSet::new(),
            reg_map: HashMap::new(),
            func_stack_info: HashMap::new(),
            child_protos: HashMap::new(),
            instr_count: 0,
        }
    }

    pub fn global_scan(&mut self, module: &IRModule) {
        for func in &module.functions {
            self.instr_count = 0;
            self.scan_lifetimes(func);
            self.allocate_registers(func);
        }
    }

    fn scan_lifetimes(&mut self, func: &ir::IRFunction) {
        for (_, &slot_id) in &func.local_variables {
            self.record_def(&func.name, VarKind::Slot(slot_id), true, None);
        }

        for block in &func.basic_blocks {
            for instr in &block.instructions {
                self.instr_count += 1;
                self.process_instr(&func.name, instr);
            }
            self.instr_count += 1;
            self.process_terminator(&func.name, &block.terminator);
        }
    }

    fn allocate_registers(&mut self, func: &ir::IRFunction) {
        let func_name = &func.name;

        let mut num_slots = 0;
        for ((f, kind), _) in &self.lifetimes {
            if f == func_name {
                if let VarKind::Slot(slot_id) = kind {
                    self.reg_map.insert((f.clone(), kind.clone()), *slot_id);
                    num_slots = num_slots.max(slot_id + 1);
                }
            }
        }

        let mut temps: Vec<_> = self.lifetimes.iter()
            .filter(|((f, kind), _)| f == func_name && matches!(kind, VarKind::Reg(_)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        temps.sort_by_key(|(_, lt)| lt.start);

        let mut active: Vec<((String, VarKind), Lifetime, usize)> = Vec::new();
        let mut free_registers: Vec<usize> = Vec::new();

        //FIXME: 这里的寄存器分配策略必须要优化，未来可能会出现突然挂掉的情况，原因如下

        // 我们这里出现一个问题，即scanner没有考虑到call进行函数调用的时候参数会覆盖活跃寄存器的情况。
        // 我们的临时解决办法是
        // 强行改变寄存器分配的策略，增加了寄存器之间的间距，以减少 Call 指令参数覆盖到活跃寄存器的概率。
        // 但这样的策略依旧没有从根本上解决问题，因为如果函数内的活跃寄存器数量超过了预留的间距，仍然可能发生覆盖。
        // 我们不考虑复杂的窗口预留，为了安全，我们让 TEMP 变量的起始位置稍微远离 Local 变量区，给 Call 指令的参数腾出位置。
        // 合理的做法是扫描call指令的参数多少，动态调整间距或者预留空间，但这会增加实现复杂度。
        // 假设 Lua 函数调用最常见的是 1-5 个参数。
        // 我们将步进间隔设置为 4，这意味着每个 TEMP 变量后面都跟着 3 个空位。
        // 这能完美防御任何参数少于 4 个的函数调用（如 print(a, b, c)）。
        let stride = 4;
        let mut next_temp_idx = num_slots;
        let mut max_usage = num_slots;

        for (key, lt) in temps {
            active.retain(|(_, active_lt, phys_idx)| {
                if active_lt.end < lt.start {
                    free_registers.push(*phys_idx);
                    false
                } else {
                    true
                }
            });

            let phys_idx = if let Some(reused_idx) = free_registers.pop() {
                reused_idx
            } else {
                let idx = next_temp_idx;
                next_temp_idx += stride;
                idx
            };

            self.reg_map.insert(key.clone(), phys_idx);
            active.push((key, lt.clone(), phys_idx));
            max_usage = max_usage.max(phys_idx + stride);
        }

        self.func_stack_info.insert(func_name.clone(), (num_slots, max_usage + stride));
    }

    fn process_instr(&mut self, func_name: &str, instr: &IRInstruction) {
        match instr {
            IRInstruction::LoadImm { dest, value } => {
                let type_str = match value {
                    IROperand::ImmFloat(_) => "Float",
                    IROperand::ImmStr(_) => "String",
                    IROperand::ImmBool(_) => "Boolean",
                    IROperand::Nil => "Nil",
                    _ => "Dynamic",
                };
                self.record_def(func_name, VarKind::Reg(*dest), false, Some(type_str));
            }
            IRInstruction::LoadLocal { dest, src } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, src);
            }
            IRInstruction::StoreLocal { dest, dst, src } => {
                let src_type = if let IROperand::Reg(id) = src {
                    self.lifetimes.get(&(func_name.to_string(), VarKind::Reg(*id)))
                        .and_then(|lt| lt.inferred_type.clone())
                } else { None };

                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                if let IROperand::Slot(slot_id) = dst {
                    if let Some(ty) = src_type {
                        if let Some(lt) = self.lifetimes.get_mut(&(func_name.to_string(), VarKind::Slot(*slot_id))) {
                            lt.inferred_type = Some(ty);
                        }
                    }
                }
                self.record_use(func_name, dst);
                self.record_use(func_name, src);
            }
            IRInstruction::Binary { dest, src1, src2, .. } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, src1);
                self.record_use(func_name, src2);
            }
            IRInstruction::Unary { dest, src, .. } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, src);
            }
            IRInstruction::Call { dest, callee, args } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, callee);

                for arg in args {
                    self.record_use(func_name, arg);
                    if let IROperand::Reg(id) = arg {
                        let key = (func_name.to_string(), VarKind::Reg(*id));
                        if let Some(lt) = self.lifetimes.get_mut(&key) {
                            lt.end = self.instr_count + 1;
                        }
                    }
                }
                if let IROperand::Reg(id) = callee {
                    let key = (func_name.to_string(), VarKind::Reg(*id));
                    if let Some(lt) = self.lifetimes.get_mut(&key) {
                        lt.end = self.instr_count + 1;
                    }
                }
            }
            IRInstruction::LoadGlobal { dest, name } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, name);
                if let IROperand::ImmStr(s) = name { self.global_vars.insert(s.clone()); }
            }
            IRInstruction::StoreGlobal { dest, name, src } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, name);
                self.record_use(func_name, src);
            }
            IRInstruction::Drop { src } => {
                self.record_use(func_name, src);
            }
            IRInstruction::NewTable { dest, size_array, size_hash } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, Some("Table"));
                self.record_use(func_name, size_array);
                self.record_use(func_name, size_hash);
            }
            IRInstruction::SetTable { dest, table, key, value } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, table);
                self.record_use(func_name, key);
                self.record_use(func_name, value);
            }
            IRInstruction::GetTable { dest, table, key } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, table);
                self.record_use(func_name, key);
            }

            IRInstruction::IndexOf { dest, collection, index } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, collection);
                self.record_use(func_name, index);
            }
            IRInstruction::SetIndex { dest, collection, index, value } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, collection);
                self.record_use(func_name, index);
                self.record_use(func_name, value);
            }
            IRInstruction::MemberOf { dest, collection, member } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, collection);
                self.record_use(func_name, member);
            }
            IRInstruction::SetMember { dest, collection, member, value } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, None);
                self.record_use(func_name, collection);
                self.record_use(func_name, member);
                self.record_use(func_name, value);
            }

            IRInstruction::FnProto { dest, func_proto } => {
                self.record_def(func_name, VarKind::Reg(*dest), false, Some("Function"));
                self.record_use(func_name, func_proto);
            }
        }
    }

    fn record_def(&mut self, func_name: &str, var: VarKind, is_fixed: bool, type_hint: Option<&str>) {
        let key = (func_name.to_string(), var);
        let entry = self.lifetimes.entry(key).or_insert(Lifetime {
            start: self.instr_count,
            end: self.instr_count,
            is_fixed,
            inferred_type: type_hint.map(|s| s.to_string()),
        });
        entry.start = entry.start.min(self.instr_count);
        if entry.inferred_type.is_none() && type_hint.is_some() {
            entry.inferred_type = type_hint.map(|s| s.to_string());
        }
    }

    fn record_use(&mut self, func_name: &str, operand: &IROperand) {
        let var = match operand {
            IROperand::Reg(id) => Some(VarKind::Reg(*id)),
            IROperand::Slot(id) => Some(VarKind::Slot(*id)),
            _ => None,
        };
        if let Some(v) = var {
            let key = (func_name.to_string(), v);
            if let Some(lt) = self.lifetimes.get_mut(&key) {
                lt.end = lt.end.max(self.instr_count);
            }
        }
    }

    fn process_terminator(&mut self, func_name: &str, term: &IRTerminator) {
        match term {
            IRTerminator::Return(ops) => { for op in ops { self.record_use(func_name, op); } }
            IRTerminator::Branch { cond, .. } => { self.record_use(func_name, cond); }
            _ => {}
        }
    }
}
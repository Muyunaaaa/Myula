// Myula compiler IR emitter
// Created by: Yuyang Feng <mu_yunaaaa@mail.nwpu.edu.cn>
// Changelog:
// 2026-02-15: Defined the emitter data structures and core methods;
//            implemented the basic translation logic from IR instructions to bytecode, covering instruction types such as constant loading, arithmetic operations, table manipulations, and function calls;
//            introduced constant pool management and register mapping mechanisms to support efficient bytecode generation;
//            implemented handling for control flow instructions including branching and returns.
// 2026-02-17: Implemented mapping and deduplication for variable names and function identifiers within the constant pool;
//            ensured that identical string constants are stored only once;
//            correctly handled the indexing relationship between functions/variables and their corresponding strings in the constant pool.

use crate::frontend::ir::{IRFunction, IRInstruction, IROperand, IRTerminator, IRBinOp, IRUnOp};
use crate::backend::translator::scanner::{Scanner, VarKind};
use crate::common::opcode::{OpCode, UnaryOpType};
use crate::common::object::LuaValue;
use std::collections::HashMap;

pub struct BytecodeEmitter<'a> {
    func_ir: &'a IRFunction,
    scanner: &'a Scanner,
    constants: Vec<LuaValue>,
    bytecode: Vec<OpCode>,
    const_map: HashMap<LuaValue, u16>,
    var_literals: HashMap<usize, IROperand>,
}

impl<'a> BytecodeEmitter<'a> {
    pub fn new(func: &'a IRFunction, scanner: &'a Scanner) -> Self {
        Self {
            func_ir: func,
            scanner: scanner,
            constants: Vec::new(),
            bytecode: Vec::new(),
            const_map: HashMap::new(),
            var_literals: HashMap::new(),
        }
    }

    pub fn emit(mut self) -> (Vec<OpCode>, Vec<LuaValue>) {
        for block in &self.func_ir.basic_blocks {
            for instr in &block.instructions {
                self.emit_instr(instr);
            }
            self.emit_terminator(&block.terminator);
        }
        (self.bytecode, self.constants)
    }

    fn emit_instr(&mut self, instr: &IRInstruction) {
        match instr {
            IRInstruction::LoadImm { dest, value } => {
                self.var_literals.insert(*dest, value.clone());

                let d = self.get_phys_reg(VarKind::Reg(*dest));
                match value {
                    IROperand::ImmFloat(f) => {
                        let c_idx = self.add_constant(LuaValue::Number(*f));
                        self.bytecode.push(OpCode::LoadK { dest: d, const_idx: c_idx });
                    }
                    IROperand::ImmBool(b) => {
                        self.bytecode.push(OpCode::LoadBool { dest: d, value: *b });
                    }
                    IROperand::Nil => self.bytecode.push(OpCode::LoadNil { dest: d }),
                    IROperand::ImmStr(s) => {
                        let c_idx = self.add_constant(LuaValue::TempString(s.clone()));
                        self.bytecode.push(OpCode::LoadK { dest: d, const_idx: c_idx });
                    }
                    _ => {}
                }
            }

            IRInstruction::Binary { dest, src1, src2, operator } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let l = self.get_reg_index(src1);
                let r = self.get_reg_index(src2);

                match operator {
                    IRBinOp::Add => self.bytecode.push(OpCode::Add { dest: d, left: l, right: r }),
                    IRBinOp::Sub => self.bytecode.push(OpCode::Sub { dest: d, left: l, right: r }),
                    IRBinOp::Mul => self.bytecode.push(OpCode::Mul { dest: d, left: l, right: r }),
                    IRBinOp::Div => self.bytecode.push(OpCode::Div { dest: d, left: l, right: r }),
                    IRBinOp::Pow => self.bytecode.push(OpCode::Pow { dest: d, left: l, right: r }),
                    IRBinOp::Concat => self.bytecode.push(OpCode::Concat { dest: d, left: l, right: r }),
                    IRBinOp::And => self.bytecode.push(OpCode::And { dest: d, left: l, right: r }),
                    IRBinOp::Or => self.bytecode.push(OpCode::Or { dest: d, left: l, right: r }),

                    IRBinOp::Eq  => self.bytecode.push(OpCode::Eq { dest: d, left: l, right: r }),
                    IRBinOp::Neq => self.bytecode.push(OpCode::Ne { dest: d, left: l, right: r }),
                    IRBinOp::Lt  => self.bytecode.push(OpCode::Lt { dest: d, left: l, right: r }),
                    IRBinOp::Gt  => self.bytecode.push(OpCode::Gt { dest: d, left: l, right: r }),
                    IRBinOp::Leq => self.bytecode.push(OpCode::Le { dest: d, left: l, right: r }),
                    IRBinOp::Geq => self.bytecode.push(OpCode::Ge { dest: d, left: l, right: r }),
                }
            }
            IRInstruction::Unary { dest, src, operator } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let s = self.get_reg_index(src);
                let op = match operator {
                    IRUnOp::Neg => UnaryOpType::Neg,
                    IRUnOp::Not => UnaryOpType::Not,
                };
                self.bytecode.push(OpCode::UnOp { dest: d, src: s, op });
            }

            IRInstruction::GetTable { dest, table, key } |
            IRInstruction::IndexOf { dest, collection: table, index: key } |
            IRInstruction::MemberOf { dest, collection: table, member: key } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let t = self.get_reg_index(table);
                let k = self.get_reg_index(key);
                self.bytecode.push(OpCode::GetTable { dest: d, table: t, key: k });
            }

            IRInstruction::SetTable { dest, table, key, value } |
            IRInstruction::SetIndex { dest, collection: table, index: key, value } |
            IRInstruction::SetMember { dest, collection: table, member: key, value } => {
                let t = self.get_reg_index(table);
                let k = self.get_reg_index(key);
                let v = self.get_reg_index(value);
                self.bytecode.push(OpCode::SetTable { table: t, key: k, value: v });
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                self.bytecode.push(OpCode::Move { dest: d, src: v });
            }

            IRInstruction::NewTable { dest, size_array, size_hash } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let s_arr = if let IROperand::ImmFloat(f) = size_array { *f as u16 } else { 0 };
                let s_hash = if let IROperand::ImmFloat(f) = size_hash { *f as u16 } else { 0 };
                self.bytecode.push(OpCode::NewTable { dest: d, size_array: s_arr, size_hash: s_hash });
            }

            IRInstruction::FnProto { dest, func_proto } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let proto_idx = self.add_constant(LuaValue::TempString(func_proto.to_string()));
                self.bytecode.push(OpCode::FnProto { dest: d, proto_idx });
            }

            IRInstruction::Call { dest, callee, args } => {
                let r_dest = self.get_phys_reg(VarKind::Reg(*dest));
                let r_func = self.get_reg_index(callee);
                for (i, arg) in args.iter().enumerate() {
                    let r_src = self.get_reg_index(arg);
                    self.bytecode.push(OpCode::Move { dest: r_func + 1 + i as u16, src: r_src });
                }
                self.bytecode.push(OpCode::Call { func_reg: r_func, argc: args.len() as u8, retc: 1 });
                self.bytecode.push(OpCode::Move { dest: r_dest, src: r_func });
            }

            IRInstruction::LoadGlobal { dest, name } => {
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                let name_idx = match name {
                    IROperand::ImmStr(s) => self.add_constant(LuaValue::TempString(s.clone())),
                    IROperand::Reg(id) => self.get_literal_as_const(id),
                    _ => self.add_constant(LuaValue::Nil),
                };
                self.bytecode.push(OpCode::GetGlobal { dest: d, name_idx });
            }

            IRInstruction::StoreGlobal { dest, name, src } => {
                let name_idx = match name {
                    IROperand::ImmStr(s) => self.add_constant(LuaValue::TempString(s.clone())),
                    IROperand::Reg(id) => self.get_literal_as_const(id),
                    _ => self.add_constant(LuaValue::Nil),
                };

                let s = self.get_reg_index(src);
                self.bytecode.push(OpCode::SetGlobal { name_idx, src: s });
                let d = self.get_phys_reg(VarKind::Reg(*dest));
                self.bytecode.push(OpCode::Move { dest: d, src: s });
            }

            IRInstruction::LoadLocal { dest, src } => {
                if let IROperand::Slot(id) = src {
                    let d = self.get_phys_reg(VarKind::Reg(*dest));
                    let s = self.get_phys_reg(VarKind::Slot(*id));
                    self.bytecode.push(OpCode::Move { dest: d, src: s });
                }
            }

            IRInstruction::StoreLocal { dest, dst, src } => {
                if let IROperand::Slot(id) = dst {
                    let slot = self.get_phys_reg(VarKind::Slot(*id));
                    let val = self.get_reg_index(src);
                    self.bytecode.push(OpCode::Move { dest: slot, src: val });
                    let d = self.get_phys_reg(VarKind::Reg(*dest));
                    self.bytecode.push(OpCode::Move { dest: d, src: val });
                }
            }
            _ => {}
        }
    }

    fn emit_terminator(&mut self, term: &IRTerminator) {
        match term {
            IRTerminator::Return(vals) => {
                if let Some(val) = vals.first() {
                    let r = self.get_reg_index(val);
                    self.bytecode.push(OpCode::Move { dest: 0, src: r });
                    self.bytecode.push(OpCode::Return { start: 0, count: 1 });
                } else {
                    self.bytecode.push(OpCode::Return { start: 0, count: 0 });
                }
            }
            IRTerminator::Branch { cond, .. } => {
                let r_cond = self.get_reg_index(cond);
                self.bytecode.push(OpCode::Test { reg: r_cond });
                self.bytecode.push(OpCode::Jump { offset: 0 });
                self.bytecode.push(OpCode::Jump { offset: 0 });
            }
            IRTerminator::Jump(_) => {
                self.bytecode.push(OpCode::Jump { offset: 0 });
            }
            _ => {}
        }
    }

    fn get_literal_as_const(&mut self, reg_id: &usize) -> u16 {
        match self.var_literals.get(reg_id).cloned() {
            Some(IROperand::ImmStr(s)) => self.add_constant(LuaValue::TempString(s)),
            Some(IROperand::ImmFloat(f)) => self.add_constant(LuaValue::Number(f)),
            _ => self.add_constant(LuaValue::Nil),
        }
    }

    fn get_phys_reg(&self, var: VarKind) -> u16 {
        *self.scanner.reg_map.get(&(self.func_ir.name.clone(), var)).unwrap() as u16
    }

    fn get_reg_index(&self, op: &IROperand) -> u16 {
        match op {
            IROperand::Reg(id) => self.get_phys_reg(VarKind::Reg(*id)),
            IROperand::Slot(id) => self.get_phys_reg(VarKind::Slot(*id)),
            _ => 0,
        }
    }

    fn add_constant(&mut self, val: LuaValue) -> u16 {
        if let Some(&idx) = self.const_map.get(&val) {
            return idx;
        }
        let idx = self.constants.len() as u16;
        self.constants.push(val.clone());
        self.const_map.insert(val, idx);
        idx
    }
}
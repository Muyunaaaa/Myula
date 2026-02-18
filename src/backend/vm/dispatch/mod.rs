mod arithmetic;
mod access;
mod control;
mod table;
mod fn_proto;
mod compare;

use crate::backend::vm::VirtualMachine;
use crate::common::opcode::OpCode;
use crate::backend::vm::error::{VMError, ErrorKind};

impl VirtualMachine {
    pub fn execute_instruction(&mut self, instr: OpCode) -> Result<(), VMError> {
        match instr {
            OpCode::Move { dest, src } => self.handle_move(dest, src),
            OpCode::LoadK { dest, const_idx } => self.handle_loadk(dest, const_idx),
            OpCode::LoadNil { dest } => self.handle_load_nil(dest),
            OpCode::LoadBool { dest, value } => self.handle_load_bool(dest, value),

            OpCode::GetGlobal { dest, name_idx } => self.handle_get_global(dest, name_idx),
            OpCode::SetGlobal { name_idx, src } => self.handle_set_global(name_idx, src),

            OpCode::Add { dest, left, right } => self.handle_add(dest, left, right),
            OpCode::Sub { dest, left, right } => self.handle_sub(dest, left, right),
            OpCode::Mul { dest, left, right } => self.handle_mul(dest, left, right),
            OpCode::Div { dest, left, right } => self.handle_div(dest, left, right),
            OpCode::UnOp { dest, src, op } => self.handle_unary_op(dest, src, op),
            OpCode::Concat { dest, left, right } => self.handle_concat(dest, left, right),
            OpCode::And { dest, left, right } => self.handle_and(dest, left, right),
            OpCode::Or { dest, left, right } => self.handle_or(dest, left, right),

            //TODO:未来可能需要增加元表支持
            OpCode::NewTable { dest, .. } => self.handle_new_table(dest),
            OpCode::GetTable { dest, table, key } => self.handle_get_table(dest, table, key),
            OpCode::SetTable { table, key, value } => self.handle_set_table(table, key, value),

            OpCode::FnProto { dest, proto_idx } => self.handle_fn_proto(dest, proto_idx),

            OpCode::Eq { left, right } => self.handle_eq(left, right),
            OpCode::Ne { left, right } => self.handle_ne(left, right),
            OpCode::Lt { left, right } => self.handle_lt(left, right),
            OpCode::Gt { left, right } => self.handle_gt(left, right),
            OpCode::Le { left, right } => self.handle_le(left, right),
            OpCode::Ge { left, right } => self.handle_ge(left, right),

            OpCode::Test { reg } => self.handle_test(reg),
            OpCode::Jump { offset } => self.handle_jump(offset),
            OpCode::Call { func_reg, argc, retc } => self.handle_call(func_reg, argc, retc),
            OpCode::Return { start, count } => self.handle_return(start, count),

            OpCode::Halt => self.handle_halt(),


            _ => Err(self.error(ErrorKind::InternalError(format!(
                "Unsupported opcode: {:?} (Instruction not implemented)",
                instr
            )))),
        }
    }
}
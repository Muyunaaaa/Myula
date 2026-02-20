use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpType {
    Neg,
    Not, 
    Len, 
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    LoadK { dest: u16, const_idx: u16 },
    LoadNil { dest: u16 },
    LoadBool { dest: u16, value: bool },
    Move { dest: u16, src: u16 },
    
    GetGlobal { dest: u16, name_idx: u16 },
    SetGlobal { name_idx: u16, src: u16 },
    
    Add { dest: u16, left: u16, right: u16 },
    Sub { dest: u16, left: u16, right: u16 },
    Mul { dest: u16, left: u16, right: u16 },
    Div { dest: u16, left: u16, right: u16 },
    Pow { dest: u16, left: u16, right: u16 },
    Concat { dest: u16, left: u16, right: u16 }, 
    And { dest: u16, left: u16, right: u16 },    
    Or { dest: u16, left: u16, right: u16 },    

    UnOp { dest: u16, src: u16, op: UnaryOpType },

    Eq { dest: u16, left: u16, right: u16 },
    Ne { dest: u16, left: u16, right: u16 },
    Lt { dest: u16, left: u16, right: u16 },
    Gt { dest: u16, left: u16, right: u16 },
    Le { dest: u16, left: u16, right: u16 },
    Ge { dest: u16, left: u16, right: u16 },
    
    Test { reg: u16 },
    Jump { offset: i32 },
    
    NewTable { dest: u16, size_array: u16, size_hash: u16 },
    GetTable { dest: u16, table: u16, key: u16 },
    SetTable { table: u16, key: u16, value: u16 },
    
    FnProto { dest: u16, proto_idx: u16 }, 
    Call { func_reg: u16, argc: u8, retc: u8 },
    Push { src: u16 },
    Return { start: u16, count: u8 },

    Halt,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::LoadK { dest, const_idx } => write!(f, "LOADK    R{} K{}", dest, const_idx),
            OpCode::LoadNil { dest } => write!(f, "LOADNIL  R{}", dest),
            OpCode::LoadBool { dest, value } => write!(f, "LOADBOOL R{} {}", dest, value),
            OpCode::Move { dest, src } => write!(f, "MOVE     R{} R{}", dest, src),
            OpCode::GetGlobal { dest, name_idx } => write!(f, "GETGLOBAL R{} K{}", dest, name_idx),
            OpCode::SetGlobal { name_idx, src } => write!(f, "SETGLOBAL K{} R{}", name_idx, src),
            OpCode::Add { dest, left, right } => write!(f, "ADD      R{} R{} R{}", dest, left, right),
            OpCode::Sub { dest, left, right } => write!(f, "SUB      R{} R{} R{}", dest, left, right),
            OpCode::Mul { dest, left, right } => write!(f, "MUL      R{} R{} R{}", dest, left, right),
            OpCode::Div { dest, left, right } => write!(f, "DIV      R{} R{} R{}", dest, left, right),
            OpCode::Pow { dest, left, right } => write!(f, "POW      R{} R{} R{}", dest, left, right),
            OpCode::Eq { dest, left, right } => write!(f, "EQ       R{} R{} R{}", dest, left, right),
            OpCode::Ne { dest, left, right } => write!(f, "NE       R{} R{} R{}", dest, left, right),
            OpCode::Lt { dest, left, right } => write!(f, "LT       R{} R{} R{}", dest, left, right),
            OpCode::Gt { dest, left, right } => write!(f, "GT       R{} R{} R{}", dest, left, right),
            OpCode::Le { dest, left, right } => write!(f, "LE       R{} R{} R{}", dest, left, right),
            OpCode::Ge { dest, left, right } => write!(f, "GE       R{} R{} R{}", dest, left, right),
            OpCode::UnOp { dest, src, op } => write!(f, "UNOP     R{} R{} {:?}", dest, src, op),
            OpCode::NewTable { dest, size_array, size_hash } => write!(f, "NEWTABLE R{} {} {}", dest, size_array, size_hash),
            OpCode::GetTable { dest, table, key } => write!(f, "GETTABLE R{} R{} R{}", dest, table, key),
            OpCode::SetTable { table, key, value } => write!(f, "SETTABLE R{} R{} R{}", table, key, value),
            OpCode::Call { func_reg, argc, retc } => write!(f, "CALL     R{} {} {}", func_reg, argc, retc),
            OpCode::Push { src } => write!(f, "PUSH     R{}", src),
            OpCode::Return { start, count } => write!(f, "RETURN   R{} {}", start, count),
            OpCode::Jump { offset } => write!(f, "JUMP     {}", offset),
            OpCode::Test { reg } => write!(f, "TEST     R{}", reg),
            OpCode::FnProto { dest, proto_idx } => write!(f, "FNPROTO  R{} K{}", dest, proto_idx),
            OpCode::Concat { dest, left, right } => { write!(f, "CONCAT   R{} R{} R{}", dest, left, right) },
            OpCode::Halt => write!(f, "HALT"),
            _ => write!(f, "{:?}", self), 
        }
    }
}
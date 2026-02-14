#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    /* --- 基础加载 --- */
    /// R(A) := K(Bx)
    /// 用于加载：数字、字符串、以及函数原型(FnProto)
    LoadK { dest: u16, const_idx: u16 },
    /// R(A) := Nil
    LoadNil { dest: u16 },
    /// R(A) := bool
    LoadBool { dest: u16, value: bool },
    /// R(A) := R(B)
    Move { dest: u16, src: u16 },

    /* --- 变量访问 --- */
    /// R(A) := Global[K(Bx)]
    GetGlobal { dest: u16, name_idx: u16 },
    /// Global[K(Bx)] := R(A)
    SetGlobal { name_idx: u16, src: u16 },

    /* --- 算术与逻辑运算 --- */
    /// 三地址：R(A) := R(B) op R(C)
    Add { dest: u16, left: u16, right: u16 },
    Sub { dest: u16, left: u16, right: u16 },
    Mul { dest: u16, left: u16, right: u16 },
    Div { dest: u16, left: u16, right: u16 },
    Mod { dest: u16, left: u16, right: u16 }, // 取模
    Pow { dest: u16, left: u16, right: u16 }, // 幂运算 (Lua常见)

    /// 一元运算：R(A) := op R(B)
    /// 对应 IRUnOp (Neg, Not, Len)
    UnOp { dest: u16, src: u16, op: UnaryOpType },

    /* --- 比较与分支 (用于实现 Branch) --- */
    /// 如果 R(A) == R(B) 不成立，则 PC++ (跳过下一条指令)
    Eq { left: u16, right: u16 },
    Ge { left: u16, right: u16 },
    Gt { left: u16, right: u16 },
    Lt { left: u16, right: u16 },
    Le { left: u16, right: u16 },
    Ne { left: u16, right: u16 },

    /// R(A) 为逻辑假(nil/false)时，PC++
    Test { reg: u16 },
    /// 无条件跳转
    Jump { offset: i32 },

    /* --- 表操作 (包含 IR 的 Member/Index 快速路径) --- */
    /// R(A) := {}
    NewTable { dest: u16, narr: u16, nrec: u16 },
    /// R(A)[R(B)] := R(C)
    SetTable { table: u16, key: u16, val: u16 },
    /// R(A) := R(B)[R(C)]
    GetTable { dest: u16, table: u16, key: u16 },

    /// 优化路径：R(A) := R(B)["Member"]
    /// 对应 IR 的 MemberOf，其中 const_idx 指向常量池中的字符串
    GetMember { dest: u16, table: u16, const_idx: u16 },
    SetMember { table: u16, const_idx: u16, val: u16 },

    /* --- 函数调用 --- */
    /// R(A) = R(func)(args...)
    Call { func_reg: u16, argc: u8, retc: u8 },
    /// 从 R(start) 开始返回 count 个值
    Return { start: u16, count: u8 },

    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpType {
    Neg, // -x
    Not, // not x
    Len, // #x
}
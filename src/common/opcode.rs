/*
   这里定义了所有的寄存器指令操作码（opcode）

*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    /* --- 基础加载 --- */
    /// R(A) := K(Bx)  从常量池加载数据到寄存器
    LoadK { dest: u16, const_idx: u16 },
    /// R(A) := Nil
    LoadNil { dest: u16 },
    /// R(A) := R(B)   寄存器拷贝
    Move { dest: u16, src: u16 },

    /* --- 变量访问 --- */
    /// R(A) := Global[K(Bx)]
    GetGlobal { dest: u16, name_idx: u16 },
    /// Global[K(Bx)] := R(A)
    SetGlobal { name_idx: u16, src: u16 },
    /// R(A) := Stack[B]  (映射你的 LoadLocal)
    GetLocal { dest: u16, slot: u16 },
    /// Stack[A] := R(B)  (映射你的 StoreLocal)
    SetLocal { slot: u16, src: u16 },

    /* --- 算术运算 (三地址模式) --- */
    /// R(A) := R(B) op R(C)
    Add { dest: u16, left: u16, right: u16 },
    Sub { dest: u16, left: u16, right: u16 },
    Mul { dest: u16, left: u16, right: u16 },
    Div { dest: u16, left: u16, right: u16 },
    /// 比较运算
    Eq { left: u16, right: u16 },
    Ge { left: u16, right: u16 },
    Lt { left: u16, right: u16 },

    /* --- 控制流 --- */
    /// 无条件跳转 (相对偏移)
    Jump { offset: i32 },
    /// 如果 R(A) 为假则跳过下一条 Jump 指令 (实现 Branch)
    Test { reg: u16, is_true: bool },

    /* --- 函数与表 --- */
    Call { func: u16, argc: u8, retc: u8 },
    Return { start: u16, count: u8 },
    NewTable { dest: u16, narr: u16, nrec: u16 },
    SetTable { table: u16, key: u16, val: u16 },
    GetTable { dest: u16, table: u16, key: u16 },

    Halt,
}
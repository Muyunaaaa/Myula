
#[derive(Debug, Clone)]
pub enum ErrorKind {
    // 类型错误：例如 1 + "a"
    TypeError(String),
    // 变量错误：访问未定义的全局变量
    UndefinedVariable(String),
    // 调用错误：尝试调用一个非函数类型
    InvalidCall(String),
    // 算术错误：除以 0 等
    ArithmeticError(String),
    // 栈溢出：递归太深
    StackOverflow,
    // 内存溢出：GC 后仍无法分配
    OutOfMemory,
    // 内部错误：OpCode 损坏或 VM 实现 Bug
    InternalError(String),
}

#[derive(Debug, Clone)]
pub struct VMError {
    pub kind: ErrorKind,
    pub func_name: String,
    pub pc: usize,
    pub stack_trace: Vec<String>,
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Runtime Error: {:?}\n  at function '{}', PC: {}\n  Context: {}",
            self.kind, self.func_name, self.pc, self.get_message()
        )
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::TypeError(msg) => write!(f, "[Type Error] {}", msg),
            ErrorKind::UndefinedVariable(msg) => write!(f, "[Undefined Variable] {}", msg),
            ErrorKind::InvalidCall(msg) => write!(f, "[Invalid Call] {}", msg),
            ErrorKind::ArithmeticError(msg) => write!(f, "[Arithmetic Error] {}", msg),
            ErrorKind::StackOverflow => write!(f, "[Stack Overflow] 递归调用过深，超出栈限制"),
            ErrorKind::OutOfMemory => write!(f, "[Out Of Memory] 内存耗尽，GC 后仍无法分配足够空间"),
            ErrorKind::InternalError(msg) => write!(f, "[Internal Error] 虚拟机内部逻辑错误: {}", msg),
        }
    }
}

impl VMError {
    pub fn get_message(&self) -> String {
        match &self.kind {
            ErrorKind::TypeError(m) => m.clone(),
            ErrorKind::UndefinedVariable(v) => format!("variable '{}' is not defined", v),
            ErrorKind::InvalidCall(m) => m.clone(),
            ErrorKind::InternalError(m) => format!("[Internal] {}", m),
            _ => format!("{:?}", self.kind),
        }
    }
}
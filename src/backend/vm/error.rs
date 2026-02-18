
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
            "ExecutionException: {}\n  at function '{}' [Offset: 0x{:04X}]",
            self.get_message(),
            self.func_name,
            self.pc
        )
    }
}


impl VMError {
    pub fn get_message(&self) -> String {
        match &self.kind {
            ErrorKind::TypeError(m) => self.format_with_fallback("TypeMismatchException", m),
            ErrorKind::InvalidCall(m) => self.format_with_fallback("IllegalInvocationException", m),
            ErrorKind::ArithmeticError(m) => self.format_with_fallback("ArithmeticException", m),
            ErrorKind::InternalError(m) => self.format_with_fallback("InternalExecutionException", m),

            ErrorKind::UndefinedVariable(v) => {
                format!("UnresolvedSymbolException: reference to undefined variable '{}'", v)
            }

            ErrorKind::StackOverflow => "StackOverflowError: call stack depth limit exceeded".into(),
            ErrorKind::OutOfMemory => "OutOfMemoryError: heap exhaustion during allocation".into(),
        }
    }

    fn format_with_fallback(&self, exception_name: &str, message: &str) -> String {
        if message.starts_with(exception_name) {
            message.to_string()
        } else {
            format!("{}: {}", exception_name, message)
        }
    }
}
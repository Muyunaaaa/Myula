use std::fs;
use std::path::Path;
use myula::backend::translator::scanner::Scanner;
// 这里的导入路径请根据你项目的实际 crate 名修改
use myula::frontend::parser::Parser;
use myula::frontend::lexer::Lexer;
use myula::frontend::ir::IRGenerator;
use myula::backend::vm::{LogLevel, VirtualMachine};

#[test]
fn test_vm_from_lua_file() {
    // 1. 读取外部 Lua 文件
    let file_path = "./lua_tests/self/04_stack_frames.lua";
    assert!(Path::new(file_path).exists(), "测试文件不存在: {}", file_path);

    let source = fs::read_to_string(file_path)
        .expect("无法读取 Lua 测试文件");

    println!("[Test] 正在编译文件: {}", file_path);

    // 2. 前端处理：Lexer -> Parser -> AST
    let mut lexer = Lexer::new(&source);
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse();

    // 3. 中端处理：AST -> IR
    let mut ir_gen = IRGenerator::new();
    ir_gen.generate(&program);

    let mut scanner = Scanner::new();
    scanner.global_scan(&ir_gen.get_module());

    let mut vm = VirtualMachine::new();
    vm.init(&ir_gen, LogLevel::Debug, &mut scanner);

    // 5. 打印 VM 内部状态（查看生成的 OpCode 和寄存器分配）
    println!("\n--- 编译产物展示 ---");
    vm.dump_internal_state();

    // 6. 基础验证
    assert!(vm.func_meta.contains_key("_start"), "必须包含主入口 _start");

    // 如果你想看具体的指令流，可以在这里检查某个函数的指令长度
    if let Some(meta) = vm.func_meta.get("_start") {
        assert!(!meta.bytecode.is_empty(), "_start 函数指令集不能为空");
    }
}
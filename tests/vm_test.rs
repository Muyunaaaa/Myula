use myula::backend::translator::scanner::Scanner;
use myula::backend::vm::{LogLevel, VirtualMachine};
use myula::frontend::ir::IRGenerator;
use myula::frontend::lexer::Lexer;
use myula::frontend::parser::Parser;
use std::fs;
use std::path::Path;
#[test]
fn test_vm_from_file() {
    let file_path = "./lua_tests/self/02_string_gc.lua";
    assert!(
        Path::new(file_path).exists(),
        "测试文件不存在: {}",
        file_path
    );

    let source = fs::read_to_string(file_path).expect("无法读取 Lua 测试文件");

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

    vm.run();
}

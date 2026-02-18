use std::fs;
use std::path::Path;
use myula::backend::vm::VirtualMachine;
use myula::frontend::ir::IRGenerator;
use myula::frontend::lexer::Lexer;
use myula::frontend::parser::Parser;
#[test]
fn test_vm_from_file() {
    let file_path = "./lua_tests/self/02_string_gc.lua";
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

    // 4. 后端初始化：IR -> Scanner -> Emitter -> VM Metadata
    let mut vm = VirtualMachine::new();
    vm.init(&ir_gen);

    vm.run();

    // 5. 打印 VM 内部状态（查看生成的 OpCode 和寄存器分配）
    println!("\n--- 编译产物展示 ---");
    vm.dump_internal_state();
}
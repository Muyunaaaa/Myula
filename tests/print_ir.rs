#[cfg(test)]
mod ir_printer {
    use std::fs;
    use std::path::Path;
    use myula::frontend::lexer::Lexer;
    use myula::frontend::parser::Parser;
    use myula::frontend::ir::IRGenerator;

    #[test]
    fn print_lua_ir_structure() {
        // ================= 配置区域 =================
        let file_path = "./lua_tests/self/02_string_gc.lua";
        // ===========================================

        if !Path::new(file_path).exists() {
            // 如果找不到文件，打印当前工作目录辅助调试
            let cwd = std::env::current_dir().unwrap();
            panic!("\n[错误] 找不到测试文件: {} \n当前工作目录: {:?}", file_path, cwd);
        }

        let lua_code = fs::read_to_string(file_path).expect("读取文件失败");

        let mut lexer = Lexer::new(&lua_code);

        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse();

        let mut ir_gen = IRGenerator::new();
        ir_gen.generate(&program);

        // 2. 获取编译后的 IR 模块
        let module = ir_gen.get_module();

        // 3. 使用你源码中定义的 to_string() 打印整个模块
        // 你在 IRModule 上实现了 to_string()，它会递归调用函数和指令的打印方法
        println!("\n{:=^80}", format!(" Myula IR 镜像: {} ", file_path));
        println!("{}", module.to_string());
        println!("{:=^80}\n", " 打印结束 ");

        let lexer = parser.get_lexer();
        let lexer_err = lexer.get_err();
        let parser_err = parser.get_err();
        let ir_err = ir_gen.get_err();

        if !lexer_err.is_empty() {
            println!("[Lexer Errors]: {:#?}", lexer_err);
        }
        if !parser_err.is_empty() {
            println!("[Parser Errors]: {:#?}", parser_err);
        }
        if !ir_err.is_empty() {
            println!("[IR Generation Errors]: {:#?}", ir_err);
        }
    }
}
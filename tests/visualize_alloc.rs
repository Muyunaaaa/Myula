#[cfg(test)]
mod register_allocation_visualizer {
    use std::fs;
    use std::path::Path;
    use myula::frontend::lexer::Lexer;
    use myula::frontend::parser::Parser;
    use myula::frontend::ir::IRGenerator;
    use myula::backend::translator::scanner::{Scanner, VarKind}; // 导入 VarKind

    #[test]
    fn test_lua_file_allocation_visualization() {
        // ================= 配置区域 =================
        let file_path = "lua_tests/deep_test.lua";
        // ===========================================

        if !Path::new(file_path).exists() {
            panic!("\n[错误] 找不到测试文件: {}\n请确保在该路径下创建了测试脚本。", file_path);
        }

        let lua_code = fs::read_to_string(file_path).expect("读取文件内容失败");

        println!("\n{:=^100}", format!(" 深度分析源文件: {} ", file_path));

        let mut lexer = Lexer::new(&lua_code);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse();

        let mut ir_gen = IRGenerator::new();
        ir_gen.generate(&program);

        let mut scanner = Scanner::new();
        scanner.global_scan(&ir_gen);

        print_detailed_report(&scanner);
    }

    fn print_detailed_report(scanner: &Scanner) {
        let mut funcs: Vec<String> = scanner.func_stack_info.keys().cloned().collect();
        funcs.sort();

        if funcs.is_empty() {
            println!("警告: 未在 IR 中检测到任何函数定义。");
            return;
        }

        for func in funcs {
            let (num_locals, max_stack) = scanner.func_stack_info.get(&func).unwrap();

            println!("\n▶ 函数标识符: [{}]", func);
            println!("  内存布局架构: [{} 个局部变量槽位] [最大虚拟机栈深度: {}]", num_locals, max_stack);
            println!("{:-<100}", "");
            println!("{:<15} | {:<8} | {:<12} | {:<10} | {:<15} | {:<10}",
                     "寄存器/槽位", "种类", "推断类型", "物理索引", "生命周期(PC)", "策略");
            println!("{:-<100}", "");

            // 1. 提取该函数的所有变量定义
            let mut vars: Vec<_> = scanner.lifetimes.iter()
                .filter(|((f, _), _)| f == &func)
                .collect();

            // 2. 排序逻辑：Slot 优先 (按索引)，Reg 随后 (按起始 PC)
            vars.sort_by(|((_, kind_a), lt_a), ((_, kind_b), lt_b)| {
                match (kind_a, kind_b) {
                    (VarKind::Slot(id_a), VarKind::Slot(id_b)) => id_a.cmp(id_b),
                    (VarKind::Slot(_), VarKind::Reg(_)) => std::cmp::Ordering::Less,
                    (VarKind::Reg(_), VarKind::Slot(_)) => std::cmp::Ordering::Greater,
                    (VarKind::Reg(_), VarKind::Reg(_)) => lt_a.start.cmp(&lt_b.start),
                }
            });

            for ((_, kind), lt) in vars {
                let p_idx = scanner.reg_map.get(&(func.clone(), kind.clone()))
                    .expect("致命错误: 丢失寄存器映射关系");

                // 格式化显示名称
                let name = match kind {
                    VarKind::Reg(id) => format!("%{}", id),
                    VarKind::Slot(id) => format!("%local_{}", id),
                };

                let kind_str = if lt.is_fixed { "LOCAL" } else { "TEMP" };
                let ty = lt.inferred_type.as_deref().unwrap_or("Dynamic");
                let strategy = if lt.is_fixed { "Fixed Slot" } else { "Reusable" };

                println!("{:<15} | {:<8} | {:<12} | R[{:<7}] | {:>3} -> {:<8} | {:<10}",
                         name, kind_str, ty, p_idx, lt.start, lt.end, strategy);
            }
        }
        println!("\n{:=^100}\n", " 分析任务圆满完成 ");
    }
}
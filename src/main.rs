use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use myula::backend::translator::scanner::{Scanner, VarKind};
use myula::backend::vm::{VirtualMachine, LogLevel};
use myula::frontend::lexer::Lexer;

#[derive(Parser)]
#[command(name = "myulac")]
#[command(version = "1.0")]
#[command(author = "Yuyang Feng && Zimeng Li")]
#[command(about = "Myula: A high-performance unified Lua compiler and VM", long_about = None)]
struct Cli {
    input: PathBuf,

    #[arg(short, long, value_enum, default_value_t = LogLevel::Release)]
    mode: LogLevel,
}

struct TraceGuard<'a> {
    mode: LogLevel,
    ir_gen: &'a myula::frontend::ir::IRGenerator,
    scanner: &'a Scanner,
    vm_ptr: *const VirtualMachine,
}

impl<'a> Drop for TraceGuard<'a> {
    fn drop(&mut self) {
        if self.mode == LogLevel::Trace {
            let vm_ref = unsafe { &*self.vm_ptr };

            println!("\n{:^105}", "*************************************************************************");
            println!("{:^105}", "MYULA COMPILER DIAGNOSTIC TRACE (AUTO-DUMP)");
            println!("{:^105}", "*************************************************************************");

            print_ir_report(self.ir_gen);
            print_scanner_report(self.scanner);
            print_emitter_report(vm_ref);

            println!("\n{:^105}\n", "--- END OF TRACE DATA ---");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let file_path = &cli.input;

    if !file_path.exists() {
        eprintln!("[Error] Source file not found: {}", file_path.display());
        std::process::exit(1);
    }

    let source = fs::read_to_string(file_path).expect(&format!(
        "Critical: Failed to read source file at {}",
        file_path.display()
    ));

    if cli.mode != LogLevel::Release {
        println!("[Myula] Compiling: {}", file_path.display());
    }

    let mut lexer = Lexer::new(&source);
    let mut parser = myula::frontend::parser::Parser::new(&mut lexer);
    let program = parser.parse();

    let mut ir_gen = myula::frontend::ir::IRGenerator::new();
    ir_gen.generate(&program);

    let mut scanner = Scanner::new();
    scanner.global_scan(&ir_gen.get_module());

    let mut vm = VirtualMachine::new();
    vm.init(&ir_gen, cli.mode, &mut scanner);

    let _guard = TraceGuard {
        mode: cli.mode,
        ir_gen: &ir_gen,
        scanner: &scanner,
        vm_ptr: &vm as *const VirtualMachine,
    };

    if cli.mode != LogLevel::Release {
        println!("--- [VM Execution Start] ---");
    }

    vm.run();

    if cli.mode != LogLevel::Release {
        println!("--- [VM Execution Finished] ---");
    }

}

fn print_ir_report(ir_gen: &myula::frontend::ir::IRGenerator) {
    let module = ir_gen.get_module();
    println!("\n{:30} {:^40} {:30}", "==========================", "IR STRUCTURE", "==========================");
    println!("{}", module.to_string());
}

fn print_emitter_report(vm: &VirtualMachine) {
    println!("\n{:30} {:^40} {:30}", "==========================", "VM FINAL STATE", "==========================");
    vm.dump_internal_state();
}

fn print_scanner_report(scanner: &Scanner) {
    let mut funcs: Vec<String> = scanner.func_stack_info.keys().cloned().collect();
    funcs.sort();

    if funcs.is_empty() {
        println!("[Warning] No function definitions detected for analysis.");
        return;
    }

    println!("\n{:30} {:^40} {:30}", "==========================", "REGISTER ALLOCATION", "==========================");

    for func in funcs {
        let (num_locals, max_stack) = scanner.func_stack_info.get(&func).unwrap();

        println!("\n▶ Subroutine: [{}]", func);
        println!("  Metrics:  [{} Locals] [{} Max Stack]", num_locals, max_stack);

        println!("{:-<105}", "");
        println!(
            "{:<15} | {:<8} | {:<12} | {:<12} | {:<15} | {:<12}",
            "Symbol", "Kind", "Type", "Phys Reg", "Lifetime (PC)", "Strategy"
        );
        println!("{:-<105}", "");

        let mut vars: Vec<_> = scanner.lifetimes.iter()
            .filter(|((f, _), _)| f == &func)
            .collect();

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
                .expect("CRITICAL: Physical register mapping missing");

            let name = match kind {
                VarKind::Reg(id) => format!("%{}", id),
                VarKind::Slot(id) => format!("%local_{}", id),
            };

            let kind_str = if lt.is_fixed { "LOCAL" } else { "TEMP" };
            let ty_str = lt.inferred_type.as_deref().unwrap_or("Dynamic");
            let strategy = if lt.is_fixed { "Fixed Slot" } else { "Reusable" };

            println!(
                "{:<15} | {:<8} | {:<12} | R[{:<9}] | {:>3} -> {:<8} | {:<12}",
                name, kind_str, ty_str, p_idx, lt.start, lt.end, strategy
            );
        }
    }
    println!("\n{:=^105}", " ALLOCATION MAP FINISHED ");
}
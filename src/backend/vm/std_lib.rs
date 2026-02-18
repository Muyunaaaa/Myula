use crate::backend::vm::error::VMError;
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

pub fn lua_builtin_print(vm: &mut VirtualMachine, base: usize, argc: usize) -> Result<usize, VMError> {
    for i in 0..argc {
        let val = vm.get_reg(base + i);
        
        let s = match val {
            LuaValue::Nil => "nil".to_string(),
            LuaValue::Boolean(b) => b.to_string(),
            LuaValue::Number(n) => n.to_string(),
            LuaValue::String(ptr) => unsafe { (*(*ptr)).data.clone() }, 
            LuaValue::Table(ptr) => format!("table: {:p}", *ptr),
            LuaValue::Function(ptr) => format!("function: {:p}", *ptr),
            LuaValue::CFunc(f) => format!("function: {:p}", f),
            _ => "unknown".to_string(),
        };

        print!("{}", s);
        
        if i < argc - 1 {
            print!("\t");
        }
    }
    
    println!();

    Ok(0)
}
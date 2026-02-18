use std::collections::HashMap;
use crate::backend::vm::error::{ErrorKind, VMError};
use crate::backend::vm::VirtualMachine;
use crate::common::object::LuaValue;

impl VirtualMachine {
    /// NEWTABLE: 创建新表 R[dest] = {}
    pub fn handle_new_table(&mut self, dest: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let new_table = crate::common::object::LuaTable {
            data: HashMap::new(),
            metatable: None,
        };

        let table_ptr = self.heap.alloc_table(new_table)
            .ok_or_else(|| self.error(ErrorKind::OutOfMemory))?;

        self.set_reg(dest as usize, LuaValue::Table(table_ptr));
        Ok(())
    }

    /// SETTABLE: R[t_reg][R[k_reg]] = R[v_reg]
    pub fn handle_set_table(&mut self, t_reg: u16, k_reg: u16, v_reg: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let table_val = self.get_reg(t_reg as usize).clone();
        let key = self.get_reg(k_reg as usize).clone();
        let val = self.get_reg(v_reg as usize).clone();

        if let LuaValue::Table(ptr) = table_val {
            if key == LuaValue::Nil {
                return Err(self.error(ErrorKind::TypeError(
                    "NullPointerException: table index is nil (illegal key)".into()
                )));
            }

            unsafe {
                (*ptr).data.data.insert(key, val);
            }
            Ok(())
        } else {
            Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: attempt to index a non-table value (actual type: '{:?}')",
                table_val
            ))))
        }
    }

    /// GETTABLE: R[dest] = R[t_reg][R[k_reg]]
    pub fn handle_get_table(&mut self, dest: u16, t_reg: u16, k_reg: u16) -> Result<(), VMError> {
        self.call_stack.last_mut().unwrap().pc += 1;
        let table_val = self.get_reg(t_reg as usize).clone();
        let key = self.get_reg(k_reg as usize).clone();

        if let LuaValue::Table(ptr) = table_val {
            let result = unsafe {
                let lua_table = &(*ptr).data; // 获取 LuaTable 引用

                match lua_table.data.get(&key) {
                    Some(v) => v.clone(),
                    None => {
                        // 如果不存在，检查元表是否存在 __index
                        // 目前默认返回 nil
                        LuaValue::Nil
                    }
                }
            };
            self.set_reg(dest as usize, result);
            Ok(())
        } else {
            Err(self.error(ErrorKind::TypeError(format!(
                "TypeMismatchException: attempt to perform property lookup on a non-table value (actual type: '{:?}')",
                table_val
            ))))
        }
    }

    // fn get_metamethod(&self, obj: &LuaValue, event: &str) -> Option<LuaValue> {
    //     if let LuaValue::Table(ptr) = obj {
    //         unsafe {
    //             // 1. 获取元表
    //             let mt_ptr = (*ptr).data.metatable?;
    //             // 2. 在元表的 data (HashMap) 中寻找事件名（如 "__add"）
    //             // 注意：这里需要将字符串转为 LuaValue 进行查找
    //             let key = LuaValue::TempString(event.to_string());
    //             (*mt_ptr).data.data.get(&key).cloned()
    //         }
    //     } else {
    //         None
    //     }
    // }
}
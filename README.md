# Myula

Myula is an experimental Lua 1.1 partial-compatible Lua runtime with future language features and a SSA IR.

![myula](./img/logo.svg)

To try it out, go

- **Download**: Visit the [Releases](https://github.com/Muyunaaaa/Myula/releases) page.

- **Run**: After downloading the binary for your system, execute the following in your terminal:
  
  ```bash
  ./myula --help
  ```
  
  to view usage instructions.

## Syntax Support

| **Category**     | **Feature**                 | **Status** | **Technical Notes**                                    |
| ---------------- | --------------------------- | ---------- | ------------------------------------------------------ |
| **Basic Syntax** | Dynamic Typing (`LuaValue`) | ✅          | Supports Nil, Bool, Num, String, Table                 |
|                  | Local/Global Variables      | ✅          | Fast scope-based lookup                                |
| **Expressions**  | Arithmetic/Logic            | ✅          | Includes Exponentiation (`^`) and Concatenation (`..`) |
|                  | Table Constructor           | ✅          | Supports mixed tables `{k=v, v}` and legacy `@` syntax |
| **Control Flow** | If-Then-Else                | ✅          | Full conditional branch support                        |
|                  | While / Repeat              | ✅          | Basic loop logic support                               |
|                  | For Loops                   | 🏗         | Iterator protocol under development                    |
| **Functions**    | First-class Functions       | ✅          | Supports nested definitions and first-class passing    |
|                  | Native Interop              | ✅          | Call Rust native code via `CFunc`                      |
|                  | Multi-return                | 🏗         | Refactoring `handle_return` for contiguous space       |
|                  | Closures                    | ✅          | Upvalue capture logic has been implemented!            |

## Authors

- **Zimeng Li**
  - Email: [zimengli@mail.nwpu.edu.cn](mailto:zimengli@mail.nwpu.edu.cn)
  - GitHub: [konakona418](https://github.com/konakona418/)
- **Yuyang Feng**
  - Email: [mu_yunaaaa@mail.nwpu.edu.cn](mailto:mu_yunaaaa@mail.nwpu.edu.cn)
  - GitHub: [Muyunaaaa](https://github.com/Muyunaaaa)

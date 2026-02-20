# Myula

Myula is a high-performance embedded scripting language implementation written in Rust. Drawing inspiration from the elegance of Lua, it introduces significant innovations in virtual machine architecture. By combining a **Register-based Instruction Set Architecture (ISA)** with a **Global Linear Stack**, Myula provides an execution environment characterized by an extremely low memory footprint and high efficiency.

![myula](https://github.com/user-attachments/assets/01d6ab69-6300-467c-ad2a-1460766d8a27)

## 📦 Installation & Usage

Myula is designed to provide an "out-of-the-box" experience with zero external dependencies.

- **Download**: Visit the [Releases](https://github.com/Muyunaaaa/Myula/releases) page.

- **Run**: After downloading the binary for your system, execute the following in your terminal:

  Bash

  ```bash
  ./myula --help
  ```

  to view usage instructions.

## 📖 Summary

Myula utilizes an integrated compilation and execution pipeline, allowing it to run source code directly without generating intermediate files (bytecode persistence is planned for the future). Leveraging the native advantages of Rust, Myula compiles into a small, independent binary that requires no complex runtime environment installation.

## 🏗 Architecture

The Myula compilation pipeline follows classic compiler design, deeply optimized for a register-based virtual machine:

1. **Lexer**: Tokenizes source code. It supports legacy Lua 1.1 syntax (such as the `@` operator) alongside modern Lua syntax.
2. **Parser**: A recursive descent parser that constructs an Abstract Syntax Tree (AST).
3. **IR Generator**: Converts the AST into a **Static Single Assignment (SSA)** Intermediate Representation for advanced analysis.
4. **Scanner**: The register allocator. It performs live-range analysis on the IR to calculate variable lifetimes and maps virtual registers to physical slots, enabling aggressive register reuse.
5. **Emitter**: The bytecode generator. Encodes the IR into binary bytecode executable by the VM based on the custom ISA.
6. **Virtual Machine**: The core engine. Implements a **Global Stack Offset Addressing** scheme, executing bytecode efficiently by sliding stack frame windows.

## 🚀 Syntax Support

| **Category**     | **Feature**                 | **Status** | **Technical Notes**                                    |
| ---------------- | --------------------------- | ---------- | ------------------------------------------------------ |
| **Basic Syntax** | Dynamic Typing (`LuaValue`) | ✅          | Supports Nil, Bool, Num, String, Table                 |
|                  | Local/Global Variables      | ✅          | Fast scope-based lookup                                |
| **Expressions**  | Arithmetic/Logic            | ✅          | Includes Exponentiation (`^`) and Concatenation (`..`) |
|                  | Table Constructor           | ✅          | Supports mixed tables `{k=v, v}` and legacy `@` syntax |
| **Control Flow** | If-Then-Else                | ✅          | Full conditional branch support                        |
|                  | While / Repeat              | ✅          | Basic loop logic support                               |
|                  | For Loops                   | 🏗          | Iterator protocol under development                    |
| **Functions**    | First-class Functions       | ✅          | Supports nested definitions and first-class passing    |
|                  | Native Interop              | ✅          | Call Rust native code via `CFunc`                      |
|                  | Multi-return                | 🏗          | Refactoring `handle_return` for contiguous space       |
|                  | Closures                    | ✅          | Upvalue capture logic in planning                      |

## 🛠 Module Features

### 1. Virtual Machine (Core Engine)

- **Global Linear Stack Model**: Breaks away from traditional independent frame arrays. All registers are physically contiguous, localized via a `base_offset`.
- **Mark-sweep GC**: Implements traditional Garbage Collection by traversing global variables, constant pools, and all active registers to reclaim memory.
- **Zero-Copy Calling Convention**: Arguments are pre-aligned at the top of the physical stack before a call, allowing the callee to take "in-place ownership" of parameters.
- **Hierarchical Upvalue Management**: Implements Lexical Scoping by capturing local variables into heap-allocated Upvalues. Supports Open-to-Closed state transition, ensuring that closures remain valid even after the owner function's stack frame is reclaimed.

### 2. Scanner (Register Allocator)

- **Live Range Analysis**: Precisely calculates the lifetime of every intermediate variable.
- **Register Reuse**: Automatically identifies non-overlapping variables and assigns them to the same physical register slot, drastically compressing the function stack size.

### 3. IR Generator & Emitter

- **SSA-based IR**: Provides the foundation for register optimization by converting AST into Static Single Assignment form.
- **Instruction Alignment Optimization**: The Emitter generates compact binary bytecode and optimizes common operations (like `Move` and `Push`) at the instruction level.

### 4. Lexer & Parser

- **Dual-Syntax Compatibility**: Supports modern Lua while maintaining compatibility with legacy Lua 1.1 features (like the `@` table constructor).
- **Error Recovery**: Collects and reports detailed error locations during parsing rather than crashing on the first encounter.

## 🛠 Build & Development

Myula is built using the standard Rust toolchain.

### 1. Prerequisites

- **Rust**: 1.75.0 or higher.
- **Cargo**: Installed automatically with Rust.

### 2. Clone & Compile

```bash
git clone https://github.com/Muyunaaaa/Myula.git
cd Myula

cargo build --release
```

Once completed, the binary will be located at `target/release/myula`.

## 👥 Authors & Contributors

- **Zimeng Li**
  - Email: [zimengli@mail.nwpu.edu.cn](mailto:zimengli@mail.nwpu.edu.cn)
  - GitHub: [konakona418](https://github.com/konakona418/)
- **Yuyang Feng**
  - Email: [mu_yunaaaa@mail.nwpu.edu.cn](mailto:mu_yunaaaa@mail.nwpu.edu.cn)
  - GitHub: [Muyunaaaa](https://github.com/Muyunaaaa)

### 第一阶段：对象系统与内存布局（地基）

**目标**：定义虚拟机能识别的所有数据类型。

1. **定义通用对象结构（TValue/Object）**：
   - 使用 `union` + `tag` 模式。
   - 包含基本类型：`NIL`, `NUMBER`, `BOOLEAN`。
   - 包含引用类型：`STRING`, `TABLE`, `FUNCTION`（这些存指针）。
2. **实现字符串池（String Interning）**：
   - 编写一个简单的哈希表，用于存储全局唯一的字符串。
   - **作用**：保证两个 `"hello"` 在内存里只有一份，方便后续进行 $O(1)$ 级别的地址比较。
3. **实现核心数据结构 Table**：
   - 这是 Lua 的灵魂。你需要用 C 写一个能够自动扩容的哈希表。
   - 提供 `table_get` 和 `table_set` 接口。

------

### 第二阶段：虚拟机基础设施（水电路）

**目标**：搭建执行指令的硬件模拟环境。

1. **构建数据栈（The Data Stack）**：
   - 创建一个 `Object` 数组。
   - 实现 `push()` 和 `pop()`。
   - **注意**：暂时不需要考虑复杂的函数调用栈（Call Stack），先保证能跑通 `1 + 2`。
2. **编写指令分发器大循环（The Main Loop）**：
   - 定义最初始的 `OpCode` 枚举（如 `OP_MOVE`, `OP_LOADK`, `OP_ADD`, `OP_SETGLOBAL`, `OP_HALT`）。
   - 写出 `switch-case` 的骨架。

------

### 第三阶段：指令逻辑实现（家电安装）

**目标**：让虚拟机具备真正的计算能力。

1. **实现算术运算逻辑**：
   - 在 `switch` 的 `OP_ADD` 下写逻辑：弹出两个 `NUMBER`，相加，压回结果。
2. **实现全局环境（Global Table）**：
   - 创建一个全局单例 Table。
   - 实现 `OP_GETGLOBAL`：从常量池拿索引 $\rightarrow$ 取字符串 $\rightarrow$ 去全局表查 $\rightarrow$ 压栈。

------

### 第四阶段：反序列化与常量池构建器（接口预留）

**目标**：为对接前端的 IR 文件做好准备。

1. **编写二进制读取模块（ByteStream Reader）**：
   - 实现从文件中读取 `byte`, `int`, `double`, `string` 的工具函数。
   - 处理字节序转换（如果需要跨平台）。
2. **实现后端常量池（Constant Pool Builder）**：
   - 提供接口：`int register_constant(Object o)`。
   - 当以后反序列化 IR 时，你把 IR 里的常量全塞进去，它会吐出索引供你生成字节码。

------

### 第五阶段：后端翻译器（对接前端）

**目标**：将前端生成的 IR 映射到你的字节码上。

1. **编写 IR-to-Bytecode 转换器**：
   - **由于前端还没定 IR**，你现在可以先写一个**测试存根（Test Stub）**：
   - 手动在 C 代码里组装一组逻辑（比如模拟 IR 序列：`PUSH 10`, `STORE_GLOBAL "a"`），看你的转换器能不能生成正确的字节码数组。
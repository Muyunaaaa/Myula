-- 测试：立即数加载、算术运算、多参数 print
-- TEST SUCCESSFUL
local a = 10
local b = 20
local c = a + b
local d = c * 2 - 5
print("Basic Math Test:")
print("a + b = ", c)      -- 验证寄存器连续传递
print("Result d = ", d)   -- 验证复杂表达式 R(n) = R(n) op R(m)
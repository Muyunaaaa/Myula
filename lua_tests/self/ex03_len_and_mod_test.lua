-- Myula Arithmetic & Length Operator Test

local function test_ops()
print("--- Starting Operator Test ---")

-- 1. 测试取模运算符 %
local i = 1
local mod_count = 0
while i <= 100 do
if i % 10 == 0 then
mod_count = mod_count + 1
end
i = i + 1
end
print("Mod check (Expected 10):")
print(mod_count)

-- 2. 测试字符串长度 #
local str = "Hello Myula"
print("String length (Expected 11):")
print(#str)

-- 3. 测试 Table 数组长度 #
-- 注意：这里测试的是数组部分的连续索引长度
local my_list = {10, 20, 30, 40, 50}
print("Table length (Expected 5):")
print(#my_list)

-- 4. 混合运算压力
local result = (100 % 7) + #("abc") + #({1, 1, 1})
-- (100 % 7 = 2) + 3 + 3 = 8
print("Mixed result (Expected 8):")
print(result)

return result
end

local final_val = test_ops()
if final_val == 8 then
print("--- All Operator Tests Passed! ---")
else
print("--- Test Failed! ---")
end
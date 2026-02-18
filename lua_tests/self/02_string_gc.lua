-- 测试：字符串常量转换、动态拼接、GC 标记
local header = "Myula"
local space = " "
local version = "v2026"

-- 测试动态拼接 (Concat)
local full_name = header .. space .. version

print("String Test:")
print("Full Name:", full_name)

-- 循环创建临时字符串，测试 GC 是否触发 sweep
-- 使用 while 循环模拟：for i = 1, 100 do ... end
print("Starting GC stress test...")

local temp = ""
local i = 1
while i <= 3 do
    -- 每次拼接都会在堆上分配新字符串，老字符串会变成垃圾
    temp = "Current index: " .. i
    i = i + 1
end

print("Final Temp:", temp)

-- 也可以用 repeat until 再跑一次压力
local j = 1
repeat
    temp = "Repeat index: " .. j
    j = j + 1
until j > 3

print("Final Repeat Temp:", temp)
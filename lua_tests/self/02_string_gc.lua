-- 测试：字符串常量转换、动态拼接、GC 标记
local header = "Myula"
local space = " "
local version = "v2026"
--TODO: 需要等待 ir 实现concat
local full_name = header .. space .. version -- 触发大量 alloc_string

print("String Test:")
print("Full Name:", full_name)

-- 循环创建临时字符串，测试 GC 是否触发 sweep
local temp = ""
for i = 1, 100 do
    temp = "Current index: " .. i
end
print("Final Temp:", temp)
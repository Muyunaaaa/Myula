-- 测试：表创建、键值对存取、元表占位
-- TEST SUCCESSFUL
local t = {}
t["key"] = "value"
t[123] = 456

print("Table Test:")
print("t['key']:", t["key"])
print("t[123]:", t[123])

-- 模拟复杂表，测试 GC 递归标记 (mark_value 对 Table 的处理)
local root = { data = "root" }
root.child = { data = "child" }
root.child.parent = root -- 测试循环引用，GC 不应死循环
print("Circular Table Data:", root.child.parent.data)
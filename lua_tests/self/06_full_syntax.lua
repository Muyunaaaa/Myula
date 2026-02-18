-- 测试：递归调用（栈深度检查）、条件分支、大内存分配
-- TEST FAILED
local function factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end

print("Factorial of 5:", factorial(5))

-- 触发潜在的 OOM (如果 HARD_MEMORY_LIMIT 较小)
local big_table = {}
for i = 1, 1000 do
    big_table["key_" .. i] = "value_value_value_value_value"
end
print("Big table populated, count:", 1000)
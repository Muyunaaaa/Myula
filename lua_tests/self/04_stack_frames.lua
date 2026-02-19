-- 测试：深度调用栈、参数拷贝、RETURN 后的 PC 恢复
-- TEST FAILED
function add(x, y)
    return x + y
end

function square(n)
    return n * n
end

function complex_calc(a, b)
    local sum = add(a, b)
    return square(sum)
end

local result = complex_calc(2, 3)
print("Complex Calculation (2+3)^2 =", result) -- 应输出 25
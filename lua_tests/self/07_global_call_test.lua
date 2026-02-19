-- 测试目标：纯全局函数调用与多层嵌套栈帧
-- 注意：不使用 local function，不使用闭包，不使用循环

function multiply(x, y)
    return x * y
end

function calculate_area(radius)
    -- 这里会触发 GETGLOBAL "multiply"
    -- 然后进行平方计算：radius * radius
    local r_squared = multiply(radius, radius)
    -- 再乘以 PI (约 3)
    return multiply(r_squared, 3)
end

function start_test(input_val)
    local result = calculate_area(input_val)
    -- 返回计算结果给主程序
    return result
end

-- 主逻辑触发
final_val = start_test(5)
-- 预期结果：5 * 5 * 3 = 75
print("Test Result (Expected 75):")
print(final_val)
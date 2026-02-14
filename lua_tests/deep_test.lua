-- 多层调用与作用域隔离测试

-- 定义内部函数 (嵌套调用层级 1)
function multiplier(n,base_value)
    -- Level 2: 这里的 n 是局部变量
    local temp = n * factor
    return temp + base_value
end

function sum_factory(a, b, base_value)
    -- Level 2: 深度计算产生大量临时变量 (SSA %n)
    local res = multiplier(a, base_value) + multiplier(b, base_value)
    return res
end

function factory(base_value)
    return sum_factory(5, 10, base_value)
end

-- 全局层级调用
local final_result = factory(100)
print("Final Result: " .. final_result)
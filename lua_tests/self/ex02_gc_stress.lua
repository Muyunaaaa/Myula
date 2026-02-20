-- Myula GC & Upvalue Stress Test

local function create_adder(base)
    -- base 是被捕获的 Upvalue
    return function(n)
        return base + n
    end
end

local function stress_test()
    local i = 0
    local total_sum = 0

    -- 第一阶段：产生大量临时闭包
    print("Stage 1: Massive temporary closures...")
    while i < 10000 do
        local adder = create_adder(i)
        total_sum = total_sum + adder(1)

        -- 每 1000 次尝试通过创建一个大表来迫使分配器工作
        if i < 1000 then
            local junk = {1, 2, 3, 4, 5}
            print("  Progress: " .. i)
        end

        i = i + 1
    end
    print("Sum after Stage 1: " .. total_sum)

    -- 第二阶段：深层嵌套闭包与 Upvalue 链
    print("Stage 2: Nested Upvalue chains...")
    local function multiplier(a)
        return function(b)
            return function(c)
                return a * b * c
            end
        end
    end

    local j = 1
    local mult_sum = 0
    while j < 5000 do
        -- 这种三层嵌套会产生大量需要 GC 跟踪的 Upvalue 对象
        local m1 = multiplier(j)
        local m2 = m1(2)
        mult_sum = mult_sum + m2(3)
        j = j + 1
    end
    print("Mult sum after Stage 2: " .. mult_sum)

    return total_sum + mult_sum
end
local final_result = stress_test()
print("Final Result: " .. final_result)
print("GC Stress Test Passed!")
-- UNSUPPORTED
-- 测试多返回值的典型场景
function get_user_data()
    local name = "Alice"
    local age = 25
    local score = 98.5
    -- 按照你的 Emitter 逻辑，这里会生成一系列 MOVE 指令
    -- 将 name, age, score 分别搬运到 R0, R1, R2
    return name, age, score
end

-- 调用点
-- Scanner 必须为 n, a, s 分配连续的寄存器
-- 否则 VM 在 handle_return 写入时会覆盖掉其他变量
local n, a, s = get_user_data()

print(n)
print(a)
print(s)
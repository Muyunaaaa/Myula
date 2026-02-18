-- 测试：全局表读写、标准库注入
--TODO: 等待ir的子函数列表实现
msg = "I am a global variable"

local function test_global()
    print("Inside function:", msg)
    msg = "Global changed!"
end

test_global()
print("Outside function:", msg)
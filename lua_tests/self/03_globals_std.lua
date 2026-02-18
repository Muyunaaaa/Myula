-- 测试：全局表读写、标准库注入
-- TEST FAILED
msg = "I am a global variable"

local function test_global()
    print("Inside function:", msg)
    msg = "Global changed!"
end

test_global()
print("Outside function:", msg)
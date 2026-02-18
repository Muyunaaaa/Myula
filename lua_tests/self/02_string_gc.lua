-- TEST SUCCESSFUL
local header = "Myula"
local space = " "
local version = "v2026"
local full_name = header .. space .. version

print("String Test:")
print("Full Name:", full_name)
print("Starting GC stress test...")

local temp = ""
local i = 1
while i <= 100 do
    temp = "Current index: " .. i
    i = i + 1
end

print(temp)
print("Final Temp:", temp)

local j = 1
repeat
    temp = "Repeat index: " .. j
    j = j + 1
until j > 100

print(temp)
print("Final Repeat Temp:", temp)
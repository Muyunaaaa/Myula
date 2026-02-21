local x = 10

local function closure()
    print(x) -- Should print 10
    return function()
        print(x) -- Should print 10
        x = 20
    end
end

closure()() -- Should print 10 twice, then change x to 20
print(x) -- Should print 20
local x = 42

local function n() 
    local function nested()
        local x = 10
        local tbl = {1, 2, 3}
        return function()
            local y = 20
            return function()
                return x + y + tbl[1] + tbl[2] + tbl[3]
            end
        end
    end
    return nested
end

local nested_func = n()
local closure_func = nested_func()
print(closure_func()()) -- 36
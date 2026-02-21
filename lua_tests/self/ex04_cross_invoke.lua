local func_a, func_b

function func_a()
    print(func_b()) -- Should print "Hello from B"
    return "Hello from A"
end

function func_b()
    return "Hello from B"
end

print(func_a()) -- Should print "Hello from A"
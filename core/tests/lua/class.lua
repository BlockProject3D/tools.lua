--- @param name string
--- @param parent table
function AbstractClass(name, parent)
    local class = {}
    if not class.init then
        class.init = function(_) end
    end
    class.__name = name
    class.__index = class
    if parent then
        setmetatable(class, parent)
    end
    return class
end

--- @param name string
--- @param parent table
function Class(name, parent)
    local class = {}
    if not class.init then
        class.init = function(_) end
    end
    class.__name = name
    class.new = function(...)
        local obj = {}
        setmetatable(obj, class)
        obj:init(...)
        return obj
    end
    class.__index = class
    if parent then
        setmetatable(class, parent)
    end
    return class
end

--- @class Parent
local Parent = AbstractClass("Parent")

function Parent:value()
    return 42
end

--- @class Child
local Child = Class("Child", Parent)

function Child:init(a)
    Parent.init(self)
    self._a = a
end

function Child:value2()
    return self:value() + self._a
end

local obj = Child.new(42)
assert(obj:value2() == 84)
assert(obj:value() == 42)

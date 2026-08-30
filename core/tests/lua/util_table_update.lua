local src = {
    fileName = "fileName",
    name = "name",
    path = bp3d.files.Path.new("whatever path probably gonna fail sandbox")
}

local dst = {
    fileName = "",
    tests = {}
}

bp3d.util.table.update(dst, src)

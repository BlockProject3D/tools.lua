// Copyright (c) 2025, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use bp3d_lua::libs::lua::Options;
use bp3d_lua::vm::RootVm;

#[test]
fn test_vm_lib_lua() {
    let mut vm = RootVm::new();
    let top = vm.top();
    bp3d_lua::libs::lua::register(&mut vm, Options::new()).unwrap();
    vm.run_code::<()>(c"
        assert(bp3d.lua.name == 'bp3d-lua')
        assert(bp3d.lua.version == '1.0.0-rc.1.0.0')
        assert(#bp3d.lua.patches == 5)
        local func = bp3d.lua.loadString('return 1 + 1')
        assert(func)
        assert(func() == 2)
        local func, err = bp3d.lua.loadString('ret a + 2')
        assert(func == nil)
        assert(err == \"syntax error: [string \\\"ret a + 2\\\"]:1: '=' expected near 'a'\")
        assert(bp3d.lua.runString('return 1 + 1') == 2)
    ").unwrap();
    let err = vm.run_code::<()>(c"bp3d.lua.require \"not.existing.file\"").unwrap_err().into_runtime().unwrap();
    assert_eq!(err.msg(), "rust error: unknown source name not");
    vm.run_code::<()>(c"
        local function test()
            bp3d.lua.require \"not.existing.file\"
        end
        local flag, err = bp3d.lua.pcall(test)
        assert(not flag)
        print(err)
        assert(err ~= '')
    ").unwrap();
    assert_eq!(vm.top(), top);
}

#[test]
fn test_vm_lib_util() {
    let vm = RootVm::new();
    let top = vm.top();
    bp3d_lua::libs::util::register(&vm).unwrap();
    vm.run_code::<()>(c"
        local src = {
            a = 1,
            b = 2
        }
        local dst = {
            c = 3
        }
        bp3d.util.table.update(dst, src)
        assert(dst.a == 1)
        assert(dst.b == 2)
        assert(dst.c == 3)
        assert(bp3d.util.table.count(dst) == 3)
        assert(bp3d.util.table.count(src) == 2)
        assert(bp3d.util.table.contains(dst, 1))
        assert(bp3d.util.table.contains(dst, 2))
        assert(bp3d.util.table.contains(dst, 3))
        assert(bp3d.util.table.containsKey(dst, 'a'))
        assert(bp3d.util.table.containsKey(dst, 'b'))
        assert(bp3d.util.table.containsKey(dst, 'c'))
        local str = bp3d.util.table.tostring(dst) .. '\\n'
        assert(bp3d.util.string.contains(str, 'a: 1\\n'))
        assert(bp3d.util.string.contains(str, 'b: 2\\n'))
        assert(bp3d.util.string.contains(str, 'c: 3\\n'))
        local str = bp3d.util.table.tostring(dst)
        local tbl = bp3d.util.string.split(str, 0x0A)
        assert(#tbl == 3)
        assert(bp3d.util.table.contains(tbl, 'a: 1'))
        assert(bp3d.util.table.contains(tbl, 'b: 2'))
        assert(bp3d.util.table.contains(tbl, 'c: 3'))
    ").unwrap();
    assert_eq!(vm.top(), top);
}

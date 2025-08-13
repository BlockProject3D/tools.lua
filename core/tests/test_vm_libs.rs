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

#![cfg(all(feature = "root-vm", feature = "libs", feature = "module"))]

use bp3d_lua::libs::lua::{Lua, Module};
use bp3d_lua::libs::util::Util;
use bp3d_lua::libs::Lib;
use bp3d_lua::module::VERSION;
use bp3d_lua::vm::RootVm;

#[test]
fn test_vm_lib_lua() {
    let vm = RootVm::new();
    let top = vm.top();
    Lua::new().build().register(&vm).unwrap();
    Module::new(&[]).register(&vm).unwrap();
    vm.set_global("BP3D_LUA_CRATE_VERSION", VERSION).unwrap();
    vm.run_code::<()>(
        c"
        assert(bp3d.lua.name == 'bp3d-lua')
        assert(bp3d.lua.version == BP3D_LUA_CRATE_VERSION)
        assert(#bp3d.lua.patches == 7)
        local func = bp3d.lua.loadString('return 1 + 1')
        assert(func)
        assert(func() == 2)
        local func, err = bp3d.lua.loadString('ret a + 2')
        assert(func == nil)
        assert(err == \"syntax error: [string \\\"ret a + 2\\\"]:1: '=' expected near 'a'\")
        assert(bp3d.lua.runString('return 1 + 1') == 2)
    ",
    )
    .unwrap();
    let err = vm
        .run_code::<()>(c"bp3d.lua.require \"not.existing.file\"")
        .unwrap_err()
        .into_runtime()
        .unwrap();
    assert_eq!(err.msg(), "rust error: unknown source name not");
    vm.run_code::<()>(c"
        local function test()
            bp3d.lua.require \"not.existing.file\"
        end
        local flag, err = bp3d.lua.pcall(test)
        assert(not flag)
        print(err)
        assert(err ~= '')
    ")
    .unwrap();
    let err = vm
        .run_code::<()>(c"MODULES:load('broken', 'broken2')")
        .unwrap_err()
        .into_runtime()
        .unwrap();
    assert_eq!(
        err.msg(),
        "rust error: module error: module not found (broken)"
    );
    assert_eq!(vm.top(), top);
}

#[test]
fn test_vm_lib_util() {
    let mut vm = RootVm::new();
    let top = vm.top();
    Util.register(&mut vm).unwrap();
    vm.run_code::<()>(
        c"
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
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local utf8 = bp3d.util.utf8
        assert(utf8.fromString('abc') ~= nil)
        assert(utf8.count('abc') == 3)
        local tbl = utf8.split('a;b;c;d', ';')
        assert(#tbl == 4)
        assert(tbl[1] == 'a')
        assert(tbl[2] == 'b')
        assert(tbl[3] == 'c')
        assert(tbl[4] == 'd')
        assert(utf8.charAt('abc', 0) == 0x61)
        assert(utf8.charAt('abc', 1) == 0x62)
        assert(utf8.charAt('abc', 2) == 0x63)
        local s = '我是'
        assert(utf8.sub(s, 1) == '是')
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local tbl = { value = 42 }
        local protected = bp3d.util.table.protect(tbl)
        assert(protected.value == 42)
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local tbl = { value = 42 }
        local protected = bp3d.util.table.protect(tbl)
        protected.value = 84
    ",
    )
    .unwrap_err();
    vm.run_code::<()>(
        c"
        local src = { value = 42, adding = { a = 1 } }
        local dst = { value = 42, adding = { } }
        bp3d.util.table.update(dst, src)
        assert(dst.value == 42)
        assert(dst.adding.a == 1)
        local dst2 = { value = 42 }
        bp3d.util.table.update(dst2, src)
        assert(dst2.value == 42)
        assert(dst2.adding.a == 1)
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local src = { value = 42, adding = { a = 1 } }
        local dst = bp3d.util.table.copy(src)
        assert(dst.value == 42)
        assert(dst.adding.a == 1)
        dst.adding.b = 2
        dst.b = 84
        assert(dst.b == 84)
        assert(src.b == nil)
        assert(dst.adding.b == 2)
        assert(src.adding.b == nil)
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local list = { 1, 2, 3, 4 }
        local list2 = { 5, 6, 7, 8 }
        bp3d.util.table.concat(list, list2)
        assert(#list == 8)
        local str = bp3d.util.table.tostring(list)
        assert(str == '1: 1\\n2: 2\\n3: 3\\n4: 4\\n5: 5\\n6: 6\\n7: 7\\n8: 8')
    ",
    )
    .unwrap();
    assert_eq!(vm.top(), top);
}

#[test]
fn test_vm_lib_os_time() {
    let mut vm = RootVm::new();
    bp3d_lua::libs::os::Time.register(&mut vm).unwrap();
    vm.run_code::<()>(
        c"
        time = bp3d.os.time.nowLocal()
        time2 = bp3d.os.time.nowUtc()
    ",
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    vm.run_code::<()>(
        c"
        local function testDateTime(a, b)
            local ymd = a:getDate()
            local ymd2 = b:getDate()
            assert(ymd.year == ymd2.year)
            assert(ymd.month == ymd2.month)
            assert(ymd.day == ymd2.day)
        end
        local now2 = bp3d.os.time.nowUtc()
        local now = bp3d.os.time.nowLocal()
        if (now ~= nil and time ~= nil) then
            assert(now > time)
        end
        assert(now2 > time2)
        if (now ~= nil and time ~= nil) then
            testDateTime(now, time)
        end
        testDateTime(now2, time2)
    ",
    )
    .unwrap();
}

#[test]
fn test_vm_lib_os_time_2() {
    let mut vm = RootVm::new();
    bp3d_lua::libs::os::Time.register(&mut vm).unwrap();
    vm.run_code::<()>(c"
        local OffsetDateTime = bp3d.os.time.OffsetDateTime
        local dt = OffsetDateTime.new({year = 1900, month = 12, day = 1})
        local date = dt:getDate()
        assert(date.year == 1900)
        assert(date.month == 12)
        assert(date.day == 1)
    ").unwrap();
}

#[test]
fn test_vm_lib_os_instant() {
    let mut vm = RootVm::new();
    bp3d_lua::libs::os::Instant.register(&mut vm).unwrap();
    vm.run_code::<()>(
        c"
        instant = bp3d.os.instant.now()
    ",
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    vm.run_code::<()>(
        c"
        local diff = instant:elapsed()
        assert((diff - 0.5) < 0.2)
    ",
    )
    .unwrap();
}

#[test]
fn test_vm_lib_os() {
    let mut vm = RootVm::new();
    bp3d_lua::libs::os::Compat.register(&mut vm).unwrap();
    vm.run_code::<()>(
        c"
        clock = os.clock()
    ",
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    vm.run_code::<()>(c"
        local now = os.clock()
        assert((clock - now) < 0.1)
    ").unwrap();
    let s = vm.run_code::<&str>(c"
        return os.date('!%H:%M:%S')
    ").unwrap();
    assert!(s.contains(":"));
    assert!(!s.contains("["));
    assert!(!s.contains("]"));
}

#[test]
fn test_vm_lib_debug() {
    let mut vm = RootVm::new();
    bp3d_lua::libs::lua::Debug.register(&mut vm).unwrap();
    vm.run_code::<()>(c"
        local debug = bp3d.lua.debug
        local libs = debug.dumpLibs();
        assert(#libs == 1)
        assert(libs[1] == 'bp3d_lua::libs::lua::debug::Debug: bp3d.lua.debug')
        local classes = debug.dumpClasses();
        assert(#classes == 0)
        local stack = debug.dumpStack(0);
        assert(#stack > 0)
    ").unwrap();
}

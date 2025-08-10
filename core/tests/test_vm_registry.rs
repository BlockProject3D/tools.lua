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

#![cfg(all(feature = "root-vm", feature = "util-method"))]

use std::ffi::CStr;
use bp3d_lua::util::{LuaFunction, LuaMethod};
use bp3d_lua::vm::registry::core::Key;
use bp3d_lua::vm::registry::lua_ref::LuaRef as LiveLuaRef;
use bp3d_lua::vm::registry::types::LuaRef;
use bp3d_lua::vm::RootVm;
use bp3d_lua::vm::table::Table;
use bp3d_lua::vm::value::types::Function;

const METHODS: &CStr = c"
local obj = { ctx = 'this is a test' }

function obj:greeting()
    return 'Hello ' .. self.ctx
end

return obj
";

#[test]
fn test_vm_registry_method() {
    let mut vm = RootVm::new();
    let top = vm.top();
    let obj: Table = vm.run_code(METHODS).unwrap();
    let method = LuaMethod::create(obj, c"greeting").unwrap();
    let str: &str = method.call(&vm, ()).unwrap();
    assert_eq!(str, "Hello this is a test");
    method.delete(&vm);
    assert_eq!(vm.top(), top + 1); // 1 result
    vm.clear();
}

#[test]
fn test_vm_registry_function() {
    let vm = RootVm::new();
    let top = vm.top();
    let obj: Function = vm.run_code(c"return function() return 'Hello world' end").unwrap();
    let f = LuaFunction::create(obj);
    assert_eq!(vm.top(), top); // The function should have been popped from the stack following the
    // call to LuaFunction
    let str: &str = f.call(&vm, ()).unwrap();
    assert_eq!(str, "Hello world");
    assert_eq!(vm.top(), top + 1); // 1 result
}

#[test]
fn test_vm_registry_string() {
    let vm = RootVm::new();
    let top = vm.top();
    let r = LiveLuaRef::new(&vm, "this is a test");
    let key: Key<LuaRef<&str>> = Key::new(r);
    assert_eq!(vm.top(), top); // The string should have been popped from the stack like any normal
    // registry creation operation.
    {
        let value = key.push(&vm).get();
        assert_eq!(value, "this is a test");
    }
    // LuaRef automatically pops from the stack on drop, as simple values which are stored in the
    // registry cannot be de-allocated by luajit.
    assert_eq!(vm.top(), top);
}

#[test]
fn test_vm_registry_string_modify() {
    let vm = RootVm::new();
    let top = vm.top();
    let r = LiveLuaRef::new(&vm, "this is a test");
    let key: Key<LuaRef<&str>> = Key::new(r);
    assert_eq!(vm.top(), top);
    let mut value = key.push(&vm);
    assert_eq!(value.get(), "this is a test");
    value.set("one more test");
    assert_eq!(value.get(), "one more test");
    key.set(value);
    assert_eq!(vm.top(), top);
    {
        let value = key.push(&vm).get();
        assert_eq!(value, "one more test");
    }
    assert_eq!(vm.top(), top);
}

#[test]
fn test_vm_registry_integer() {
    let vm = RootVm::new();
    let top = vm.top();
    let r = LiveLuaRef::new(&vm, 42);
    let key: Key<LuaRef<i32>> = Key::new(r);
    {
        let value = key.push(&vm).get();
        assert_eq!(value, 42);
    }
    assert_eq!(vm.top(), top);
}

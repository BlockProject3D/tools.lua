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

#![cfg(feature = "root-vm")]

use bp3d_lua::decl_lib_func;
use bp3d_lua::vm::function::types::RFunction;
use bp3d_lua::vm::RootVm;
use bp3d_lua_codegen::{FromParam, IntoParam};
use bp3d_lua_codegen::{IntoLua, LuaType};

#[derive(FromParam, LuaType, IntoParam)]
struct Test1<'a>(&'a str, i32);

#[derive(FromParam, LuaType, IntoParam, IntoLua)]
struct Test2<'a> {
    name: &'a str,
    value: i32,
}

#[derive(FromParam, LuaType, IntoParam)]
struct TestStatic {
    value1: f32,
    value2: i32,
}

decl_lib_func! {
    fn test(test1: Test1, test2: Test2, st: TestStatic) -> String {
        format!("{} {}: {}, {}, (v1: {}, v2: {})", test1.0, test2.name, test1.1, test2.value, st.value1, st.value2)
    }
}

decl_lib_func! {
    fn test2(name: &str) -> Test2<'_> {
        Test2 { name, value: 42 }
    }
}

decl_lib_func! {
    fn test3<'a>(name: &'a str, name2: &str) -> Test2<'a> {
        println!("{}", name2);
        Test2 { name, value: 42 }
    }
}

#[test]
fn test_custom_structs_basic() {
    let vm = RootVm::new();
    let top = vm.top();
    vm.set_global(c"test", RFunction::wrap(test)).unwrap();
    vm.set_global(c"test2", RFunction::wrap(test2)).unwrap();
    vm.set_global(c"test3", RFunction::wrap(test3)).unwrap();
    let out = vm
        .run_code::<&str>(
            c"
        local test1 = { 'value', 42 }
        local test2 = { name = 'of', value = 64 }
        local st = { value1 = 42.42, value2 = 32 }
        return test(test1, test2, st)
    ",
        )
        .unwrap();
    assert_eq!(out, "value of: 42, 64, (v1: 42.42, v2: 32)");
    vm.set_global(
        c"test",
        Test2 {
            name: "whatever",
            value: 42,
        },
    )
    .unwrap();
    let out = vm.run_code::<&str>(c"return test.name").unwrap();
    assert_eq!(out, "whatever");
    let out = vm.run_code::<i32>(c"return test.value").unwrap();
    assert_eq!(out, 42);
    vm.run_code::<()>(
        c"
        local t2 = test2('test')
        assert(t2.name == 'test')
        assert(t2.value == 42)
    ",
    )
    .unwrap();
    vm.run_code::<()>(
        c"
        local t2 = test3('test42', 'test2')
        assert(t2.name == 'test42')
        assert(t2.value == 42)
    ",
    )
    .unwrap();
    assert_eq!(top + 3, vm.top())
}

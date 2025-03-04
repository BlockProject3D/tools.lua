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

use bp3d_lua::value::FromParam;
use bp3d_lua::ffi::laux::{luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_error, luaL_loadstring};
use bp3d_lua::ffi::lua::{lua_call, lua_error, lua_gettop, lua_isstring, lua_pcall, lua_pushcclosure, lua_pushfstring, lua_pushlstring, lua_pushnumber, lua_remove, lua_setfield, lua_settop, lua_tolstring, lua_type, Number, State, GLOBALSINDEX};
use bp3d_lua::vm::{Stack, Vm};

struct ValueWithDrop;
impl ValueWithDrop {
    pub fn print(&self) {
        println!("ValueWithDrop")
    }
}
impl Drop for ValueWithDrop {
    fn drop(&mut self) {
        println!("Dropping!");
    }
}

fn safe_test_c_function(name: &str, value: f64) {
    let drop = ValueWithDrop;
    drop.print();
    println!("Hello {} ({})", name, value);
}

extern "C-unwind" fn test_c_function(l: State) -> i32 {
    let stack = unsafe { Stack::wrap(l) };
    let name: &str = FromParam::from_lua(&stack);
    let value = f64::from_lua(&stack);
    safe_test_c_function(name, value);
    1
}

extern "C" fn test_error_handler(l: State) -> i32 {
    println!("An error has occured from lua VM");
    //luaL_traceback(L, L, lua_tostring(L, 1), 1);
    1
}

#[test]
fn test_vm_destructor() {
    let vm = Vm::new();
    unsafe {
        lua_pushcclosure(vm.as_ptr(), test_c_function, 0);
        lua_setfield(vm.as_ptr(), GLOBALSINDEX, c"test_c_function".as_ptr());
        assert_eq!(luaL_loadstring(vm.as_ptr(), c"return test_c_function('this is a test', 0.42)".as_ptr()), 0);
        let i = lua_pcall(vm.as_ptr(), 0, 0, 0);
        if i != 0 {
            println!("{:?}", lua_type(vm.as_ptr(), -1));
            if lua_isstring(vm.as_ptr(), -1) == 1 {
                let mut len: usize = 0;
                let s = lua_tolstring(vm.as_ptr(), -1, &mut len as _);
                let slice = std::slice::from_raw_parts(s as *const u8, len);
                println!("Error as string: {:?}", std::str::from_utf8(slice));
            }
        }
    }
}

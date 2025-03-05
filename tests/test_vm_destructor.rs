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

use bp3d_lua::vm::function::{FromParam, IntoParam};
use bp3d_lua::ffi::lua::{lua_pushcclosure, lua_setfield, State, GLOBALSINDEX};
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

fn safe_test_c_function(name: &str, value: f64) -> String {
    let drop = ValueWithDrop;
    drop.print();
    format!("Hello {} ({})", name, value)
}

extern "C-unwind" fn test_c_function(l: State) -> i32 {
    let stack = unsafe { Stack::wrap(l, 1) };
    let name: &str = FromParam::from_param(&stack);
    let value = f64::from_param(&stack);
    let res = safe_test_c_function(name, value);
    res.into_param(&stack) as _
}

#[test]
fn test_vm_destructor() {
    let mut vm = Vm::new();
    unsafe {
        lua_pushcclosure(vm.as_ptr(), test_c_function, 0);
        lua_setfield(vm.as_ptr(), GLOBALSINDEX, c"test_c_function".as_ptr());
    }
    let res = vm.run_code::<&str>(c"return test_c_function('this is a test\\xFF', 0.42)");
    assert!(res.is_err());
    let err = res.unwrap_err().into_runtime();
    assert_eq!(err.msg(), "rust error: invalid utf-8 sequence of 1 bytes from index 14");
    assert!(vm.run_code::<&str>(c"return test_c_function('this is a test', 0.42)").is_ok());
    let s = vm.run_code::<&str>(c"return test_c_function('this is a test', 0.42)").unwrap();
    assert_eq!(s, "Hello this is a test (0.42)");
    assert!(vm.run_code::<bool>(c"return test_c_function('this is a test', 0.42)").is_err());
}

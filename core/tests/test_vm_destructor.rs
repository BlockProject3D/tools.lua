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

use bp3d_lua::decl_lib_func;
use bp3d_lua::vm::RootVm;
use bp3d_lua::vm::function::types::RFunction;

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

decl_lib_func! {
    fn test_c_function(name: &str, value: f64) -> String {
        let drop = ValueWithDrop;
        drop.print();
        format!("Hello {} ({})", name, value)
    }
}

#[test]
fn test_vm_destructor() {
    let mut vm = RootVm::new();
    vm.set_global(c"test_c_function", RFunction::wrap(test_c_function)).unwrap();
    let time = std::time::Instant::now();
    let res = vm.run_code::<&str>(c"return test_c_function('this is a test\\xFF', 0.42)");
    assert!(res.is_err());
    let err = res.unwrap_err().into_runtime().unwrap();
    assert_eq!(err.msg(), "rust error: invalid utf-8 sequence of 1 bytes from index 14");
    assert!(vm.run_code::<&str>(c"return test_c_function('this is a test', 0.42)").is_ok());
    let s = vm.run_code::<&str>(c"return test_c_function('this is a test', 0.42)").unwrap();
    assert_eq!(s, "Hello this is a test (0.42)");
    assert!(vm.run_code::<bool>(c"return test_c_function('this is a test', 0.42)").is_err());
    vm.clear();
    let time = time.elapsed();
    println!("time: {:?}", time);
}

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
use bp3d_lua::vm::value::RFunction;

decl_lib_func! {
    fn dostring(vm: &Vm, code: &str) -> bp3d_lua::vm::Result<()> {
        let ret = vm.run_code::<()>(code);
        println!("Ran code: {}", code);
        ret
    }
}

#[test]
fn test_run_string() {
    let vm = RootVm::new();
    let res = vm.run_code::<()>(c"dostring('test')");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "runtime error: [string \"dostring('test')\"]:1: attempt to call global 'dostring' (a nil value)");
    vm.set_global(c"dostring", RFunction(dostring)).unwrap();
    assert!(vm.run_code::<()>(c"dostring('test')").is_err());
    assert!(vm.run_code::<()>(c"dostring('print(\"whatever 123\")')").is_ok());
    assert!(vm.run_code::<()>(c"dostring('root = 42')").is_ok());
    let val: u32 = vm.get_global("root").unwrap();
    assert_eq!(val, 42);
}

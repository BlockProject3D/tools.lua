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

use bp3d_lua::vm::value::types::{Int53, UInt53};
use bp3d_lua::vm::RootVm;

#[test]
fn test_vm_u53() {
    let vm = RootVm::new();
    let top = vm.top();
    let val: UInt53 = vm.run_code(c"return 2^53-1").unwrap();
    assert_eq!(val, UInt53::MAX);
    vm.set_global(c"UINT53_MAX", UInt53::MAX).unwrap();
    assert_eq!(
        vm.run_code::<&str>(c"return tostring(UINT53_MAX)").unwrap(),
        "9.007199254741e+15"
    );
    vm.run_code::<()>(c"assert(UINT53_MAX == 2^53-1)").unwrap();
    assert_eq!(top + 2, vm.top());
}

#[test]
fn test_vm_i53() {
    let vm = RootVm::new();
    let top = vm.top();
    let val: Int53 = vm.run_code(c"return 2^52-1").unwrap();
    assert_eq!(val, Int53::MAX);
    let val: Int53 = vm.run_code(c"return -2^52").unwrap();
    assert_eq!(val, Int53::MIN);
    vm.set_global(c"INT53_MAX", Int53::MAX).unwrap();
    vm.set_global(c"INT53_MIN", Int53::MIN).unwrap();
    assert_eq!(
        vm.run_code::<&str>(c"return tostring(INT53_MAX)").unwrap(),
        "4.5035996273705e+15"
    );
    assert_eq!(
        vm.run_code::<&str>(c"return tostring(INT53_MIN)").unwrap(),
        "-4.5035996273705e+15"
    );
    vm.run_code::<()>(c"assert(INT53_MAX == 2^52-1)").unwrap();
    vm.run_code::<()>(c"assert(INT53_MIN == -2^52)").unwrap();
    assert_eq!(top + 4, vm.top());
}

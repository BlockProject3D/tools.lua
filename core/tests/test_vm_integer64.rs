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

use bp3d_lua::ffi::lua::Type;
use bp3d_lua::vm::RootVm;
use bp3d_lua::vm::value::any::Any;
use bp3d_lua::vm::value::util::check_type_equals;

#[test]
fn test_vm_u64() {
    let vm = RootVm::new();
    let top = vm.top();
    let val: u64 = vm.run_code(c"return 2ULL^64ULL-1ULL").unwrap();
    assert!(check_type_equals(&vm, -1, Type::Cdata).is_ok());
    assert_eq!(val, u64::MAX);
    vm.set_global(c"UINT64_MAX", u64::MAX).unwrap();
    assert_eq!(vm.run_code::<&str>(c"return tostring(UINT64_MAX)").unwrap(), "18446744073709551615ULL");
    vm.run_code::<()>(c"assert(UINT64_MAX == 2ULL^64ULL-1ULL)").unwrap();
    assert_eq!(top + 2, vm.top());
}

#[test]
fn test_vm_i64() {
    let vm = RootVm::new();
    let top = vm.top();
    let val: i64 = vm.run_code(c"return 2LL^63LL-1LL").unwrap();
    assert!(check_type_equals(&vm, -1, Type::Cdata).is_ok());
    assert_eq!(val, i64::MAX);
    let val: i64 = vm.run_code(c"return -2LL^63LL").unwrap();
    assert!(check_type_equals(&vm, -1, Type::Cdata).is_ok());
    assert_eq!(val, i64::MIN);
    vm.set_global(c"INT64_MAX", i64::MAX).unwrap();
    vm.set_global(c"INT64_MIN", i64::MIN).unwrap();
    assert_eq!(vm.run_code::<&str>(c"return tostring(INT64_MAX)").unwrap(), "9223372036854775807LL");
    assert_eq!(vm.run_code::<&str>(c"return tostring(INT64_MIN)").unwrap(), "-9223372036854775808LL");
    vm.run_code::<()>(c"assert(INT64_MAX == 2LL^63LL-1LL)").unwrap();
    vm.run_code::<()>(c"assert(INT64_MIN == -2LL^63LL)").unwrap();
    assert_eq!(top + 4, vm.top());
}

#[test]
fn test_vm_i64_any() {
    let vm = RootVm::new();
    let top = vm.top();
    let val: Any = vm.run_code(c"return 2ULL^64ULL-1ULL").unwrap();
    let val2: Any = vm.run_code(c"return 2LL^63LL-1LL").unwrap();
    assert_eq!(val.ty(), Type::Cdata);
    assert_eq!(val, Any::UInt64(u64::MAX));
    assert_eq!(val2.ty(), Type::Cdata);
    assert_eq!(val2, Any::Int64(i64::MAX));
    assert_eq!(top + 2, vm.top());
}

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

use bp3d_lua::{decl_userdata, decl_userdata_mut};
use bp3d_lua::ffi::lua::Number;
use bp3d_lua::vm::RootVm;

pub struct MyInt(i64);

decl_userdata! {
    impl MyInt {
        fn tonumber(this: &MyInt) -> Number {
            this.0 as _
        }

        fn tostring(this: &MyInt) -> String {
            this.0.to_string()
        }

        fn eq(this: &MyInt, other: &MyInt) -> bool {
            this.0 == other.0
        }

        fn lt(this: &MyInt, other: &MyInt) -> bool {
            this.0 < other.0
        }

        fn gt(this: &MyInt, other: &MyInt) -> bool {
            this.0 > other.0
        }
    }
}

#[derive(Debug)]
pub struct BrokenObject;

decl_userdata_mut! {
    impl BrokenObject {
        // this should blow up at init time
        fn replace(this: &mut BrokenObject, other: &BrokenObject) -> () {
            println!("this: {:?}, other: {:?}", this, other)
        }
    }
}

pub struct BrokenObject2(pub u128);

decl_userdata! {
    impl BrokenObject2 {
    }
}

#[derive(Debug)]
pub struct BrokenObject3;

decl_userdata! {
    impl BrokenObject3 {
        fn __gc(this: &BrokenObject3) -> () {
            println!("{:?}", this);
        }
    }
}

#[test]
fn test_vm_userdata() {
    let vm = RootVm::new();
    vm.register_userdata::<MyInt>().unwrap();
    let res = vm.register_userdata::<BrokenObject>();
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: violation of the unique type rule for mutable method \"replace\"");
    let res = vm.register_userdata::<BrokenObject2>();
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: too strict alignment required (16 bytes), max is 8 bytes");
    let res = vm.register_userdata::<BrokenObject3>();
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: __gc meta-method is reserved for internal use, if you need Vm access in drop, please use LuaDrop");
    let res = vm.register_userdata::<MyInt>();
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: class name \"MyInt\" has already been registered");
}

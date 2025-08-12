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

use bp3d_lua::ffi::lua::RawNumber;
use bp3d_lua::vm::function::types::RFunction;
use bp3d_lua::vm::userdata::{AnyUserData, LuaDrop};
use bp3d_lua::vm::{RootVm, Vm};
use bp3d_lua::{decl_lib_func, decl_userdata, decl_userdata_mut};
use std::sync::Mutex;
use bp3d_lua::util::Namespace;

static MUTEX: Mutex<()> = Mutex::new(());

static mut DROP_COUNTER: i32 = 0;
static mut LUA_DROP_COUNTER: i32 = 0;

pub struct MyInt(i64);

impl LuaDrop for MyInt {
    fn lua_drop(&self, _: &Vm) {
        unsafe {
            LUA_DROP_COUNTER += 1;
        }
    }
}

impl Drop for MyInt {
    fn drop(&mut self) {
        unsafe {
            DROP_COUNTER += 1;
        }
    }
}

decl_userdata! {
    impl MyInt {
        fn tonumber(this: &MyInt) -> RawNumber {
            this.0 as _
        }

        fn tostring(this: &MyInt) -> String {
            this.0.to_string()
        }

        fn __eq(this: &MyInt, other: &MyInt) -> bool {
            this.0 == other.0
        }

        fn __lt(this: &MyInt, other: &MyInt) -> bool {
            this.0 < other.0
        }

        fn __gt(this: &MyInt, other: &MyInt) -> bool {
            this.0 > other.0
        }

        fn __add(this: &MyInt, other: &MyInt) -> MyInt {
            MyInt(this.0 + other.0)
        }
    }

    static {
        [fn new];
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

#[derive(Debug)]
pub struct BrokenObject4;

decl_userdata! {
    impl BrokenObject4 {
        fn __index(this: &BrokenObject3) -> () {
            println!("{:?}", this);
        }
    }
}

decl_lib_func! {
    fn my_int(i: i64) -> MyInt {
        MyInt(i)
    }
}

decl_lib_func! {
    fn new(i: i64) -> MyInt {
        MyInt(i)
    }
}

#[test]
fn test_vm_userdata_forgot_reg() {
    let vm = RootVm::new();
    vm.set_global(c"MyInt", RFunction::wrap(my_int)).unwrap();
    vm.run_code::<()>(c"a = MyInt(123)").unwrap();
    vm.run_code::<()>(c"b = MyInt(456)").unwrap();
    assert!(vm.run_code::<bool>(c"return a < b").is_err());
    assert!(vm.run_code::<bool>(c"return a + b").is_err());
}

#[test]
fn test_vm_userdata_error_handling() {
    let vm = RootVm::new();
    let top = vm.top();
    vm.register_userdata::<MyInt>(bp3d_lua::vm::userdata::case::Snake)
        .unwrap();
    assert_eq!(top, vm.top());
    let res = vm.register_userdata::<BrokenObject>(bp3d_lua::vm::userdata::case::Snake);
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(
        msg,
        "userdata: violation of the unique type rule for mutable method \"replace\""
    );
    assert_eq!(top, vm.top());
    let res = vm.register_userdata::<BrokenObject2>(bp3d_lua::vm::userdata::case::Snake);
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(
        msg,
        "userdata: too strict alignment required (16 bytes), max is 8 bytes"
    );
    assert_eq!(top, vm.top());
    let res = vm.register_userdata::<BrokenObject3>(bp3d_lua::vm::userdata::case::Snake);
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: __gc meta-method is reserved for internal use, if you need Vm access in drop, please use LuaDrop");
    assert_eq!(top, vm.top());
    let res = vm.register_userdata::<MyInt>(bp3d_lua::vm::userdata::case::Snake);
    assert_eq!(top, vm.top());
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(
        msg,
        "userdata: class name \"MyInt\" has already been registered"
    );
    assert_eq!(top, vm.top());
    let res = vm.register_userdata::<BrokenObject4>(bp3d_lua::vm::userdata::case::Snake);
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert_eq!(msg, "userdata: __index meta-method is required to be surrendered to luaL_newmetatable, it is impossible to bind custom code to __index");
    assert_eq!(top, vm.top());
}

fn test_vm_userdata_base(vm: &Vm) {
    unsafe {
        DROP_COUNTER = 0;
        LUA_DROP_COUNTER = 0;
    }
    let top = vm.top();
    vm.register_userdata::<MyInt>(bp3d_lua::vm::userdata::case::Snake)
        .unwrap();
    assert_eq!(top, vm.top());
    vm.set_global(c"MyInt", RFunction::wrap(my_int)).unwrap();
    assert_eq!(top, vm.top());
    vm.run_code::<()>(c"a = MyInt(123)").unwrap();
    vm.run_code::<()>(c"b = MyInt(456)").unwrap();
    vm.run_code::<()>(c"c = MyInt(456)").unwrap();
    assert_eq!(vm.run_code::<bool>(c"return a == b").unwrap(), false);
    assert_eq!(vm.run_code::<bool>(c"return b == c").unwrap(), true);
    assert_eq!(vm.run_code::<bool>(c"return a < b").unwrap(), true);
    assert_eq!(vm.run_code::<bool>(c"return b > a").unwrap(), true);
    assert_eq!(vm.run_code::<&MyInt>(c"return a + b").unwrap().0, 579);
    assert_eq!(
        vm.run_code::<&str>(c"return (a + b):tostring()").unwrap(),
        "579"
    );
    assert_eq!(
        vm.run_code::<RawNumber>(c"return (a + b):tonumber()")
            .unwrap(),
        579.0
    );
    assert_eq!(
        vm.run_code::<RawNumber>(c"return a.tonumber(b)").unwrap(),
        456.0
    );
    assert_eq!(top + 8, vm.top());
}

fn test_vm_userdata_base2(vm: &Vm) {
    unsafe {
        DROP_COUNTER = 0;
        LUA_DROP_COUNTER = 0;
    }
    let top = vm.top();
    {
        let mut namespace = Namespace::new(vm, "_G").unwrap();
        namespace.add_userdata::<MyInt>("MyInt", bp3d_lua::vm::userdata::case::Snake).unwrap();
    }
    assert_eq!(top, vm.top());
    vm.run_code::<()>(c"a = MyInt.new(123)").unwrap();
    vm.run_code::<()>(c"b = MyInt.new(456)").unwrap();
    vm.run_code::<()>(c"c = MyInt.new(456)").unwrap();
    assert_eq!(vm.run_code::<bool>(c"return a == b").unwrap(), false);
    assert_eq!(vm.run_code::<bool>(c"return b == c").unwrap(), true);
    assert_eq!(vm.run_code::<bool>(c"return a < b").unwrap(), true);
    assert_eq!(vm.run_code::<bool>(c"return b > a").unwrap(), true);
    assert_eq!(vm.run_code::<&MyInt>(c"return a + b").unwrap().0, 579);
    assert_eq!(
        vm.run_code::<&str>(c"return (a + b):tostring()").unwrap(),
        "579"
    );
    assert_eq!(
        vm.run_code::<RawNumber>(c"return (a + b):tonumber()")
            .unwrap(),
        579.0
    );
    assert_eq!(
        vm.run_code::<RawNumber>(c"return a.tonumber(b)").unwrap(),
        456.0
    );
    assert_eq!(top + 8, vm.top());
}

#[test]
fn test_vm_userdata() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        let top = vm.top();
        test_vm_userdata_base(&vm);
        assert_eq!(top + 8, vm.top());
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_security1() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base(&vm);
        vm.run_code::<()>(c"getmetatable(a).__gc = function() print(\"Lua has hacked Rust\") end")
            .unwrap_err();
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_security2() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base(&vm);
        vm.run_code::<()>(c"a.__gc = function() print(\"Lua has hacked Rust\") end")
            .unwrap_err();
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_security3() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base(&vm);
        vm.run_code::<()>(c"setmetatable(a, nil)").unwrap_err();
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_security4() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base(&vm);
        vm.run_code::<()>(
            c"
            local func = a.tonumber
            local tbl = {}
            tbl.tonumber = func
            tbl:tonumber()
        ",
        )
        .unwrap_err();
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_security5() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base(&vm);
        vm.run_code::<()>(
            c"
            rawset(a, '__gc', nil)
        ",
        )
        .unwrap_err();
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

#[test]
fn test_vm_userdata_call_method() {
    let _guard = MUTEX.lock();
    let vm = RootVm::new();
    let top = vm.top();
    vm.register_userdata::<MyInt>(bp3d_lua::vm::userdata::case::Snake).unwrap();
    vm.set_global("MY_INT", MyInt(123456)).unwrap();
    let ud: AnyUserData = vm.get_global("MY_INT").unwrap();
    let val: &str = ud.call_method("tostring", ()).unwrap();
    let val2: RawNumber = ud.call_method("tonumber", ()).unwrap();
    assert_eq!(val, "123456");
    assert_eq!(val2, 123456.0);
    assert_eq!(vm.top(), top + 3);
}

#[test]
fn test_vm_userdata_display() {
    let _guard = MUTEX.lock();
    let vm = RootVm::new();
    let top = vm.top();
    vm.register_userdata::<MyInt>(bp3d_lua::vm::userdata::case::Snake).unwrap();
    vm.set_global("MY_INT", MyInt(123456)).unwrap();
    let ud: AnyUserData = vm.get_global("MY_INT").unwrap();
    let s = ud.to_string();
    assert_eq!(s, "MyInt(123456)");
    assert_eq!(vm.top(), top + 1);
}

#[test]
fn test_vm_userdata_statics() {
    let _guard = MUTEX.lock();
    {
        let vm = RootVm::new();
        test_vm_userdata_base2(&vm);
        vm.run_code::<()>(c"
            MyInt.__gc = function() print(\"Lua has hacked Rust\") end
            MyInt.__metatable = function() print(\"Lua has hacked Rust\") end
        ").unwrap();
        let ud: AnyUserData = vm.get_global("a").unwrap();
        let s = ud.to_string();
        assert_eq!(s, "MyInt(123)");
    }
    assert_eq!(unsafe { DROP_COUNTER }, 6);
    assert_eq!(unsafe { LUA_DROP_COUNTER }, 6);
}

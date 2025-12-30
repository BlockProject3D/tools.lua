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

#![cfg(all(feature = "root-vm", feature = "util-namespace"))]

use bp3d_lua::decl_closure;
use bp3d_lua::util::Namespace;
use bp3d_lua::vm::closure::context::{CellMut, ContextMut};
use bp3d_lua::vm::closure::types::RClosure;
use bp3d_lua::vm::RootVm;
use std::cell::Cell;

struct TestContext {
    value: i32,
    value3: Vec<u64>,
}

decl_closure! {
    fn context_set_value |ctx: ContextMut<TestContext>| (val: i32) -> () {
        let mut ctx = ctx.borrow();
        ctx.value = val;
    }
}

decl_closure! {
    fn context_push |ctx: ContextMut<TestContext>| (val: u64) -> () {
        let mut ctx = ctx.borrow();
        ctx.value3.push(val);
    }
}

decl_closure! {
    fn context_pop |ctx: ContextMut<TestContext>| () -> Option<u64> {
        let mut ctx = ctx.borrow();
        ctx.value3.pop()
    }
}

decl_closure! {
    fn test |upvalue: &str| (val: f32) -> String {
        format!("{}: {}", upvalue, val)
    }
}

#[test]
fn test_vm_fast_closure() {
    let vm = RootVm::new();
    let top = vm.top();
    vm.set_global(c"test", test("this is a test")).unwrap();
    assert_eq!(top, vm.top());
    let s: &str = vm.run_code(c"return test(42.42)").unwrap();
    assert_eq!(s, "this is a test: 42.42");
}

#[test]
fn test_vm_rust_closure() {
    let vm = RootVm::new();
    let top = vm.top();
    let closure = RClosure::from_rust(&vm, |val: f32| format!("this is a test: {}", val));
    vm.set_global(c"test", closure).unwrap();
    assert_eq!(top, vm.top());
    let s: &str = vm.run_code(c"return test(42.42)").unwrap();
    assert_eq!(s, "this is a test: 42.42");
}

#[test]
fn test_vm_context() {
    let vm = RootVm::new();
    let top = vm.top();
    let ctx = ContextMut::new(&vm);
    {
        let mut namespace = Namespace::new(&vm, "context").unwrap();
        namespace
            .add([
                ("push", context_push(ctx)),
                ("pop", context_pop(ctx)),
                ("set_value", context_set_value(ctx)),
            ])
            .unwrap();
    }
    assert_eq!(top, vm.top());
    let res = vm.run_code::<()>(c"context.set_value(42)");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().into_runtime().unwrap().msg(),
        "[string \"context.set_value(42)\"]:1: Context is not available in this function."
    );
    let mut obj = TestContext {
        value: 0,
        value3: vec![],
    };
    let mut cell = CellMut::new(ctx);
    {
        let _obj = cell.bind(&mut obj);
        vm.run_code::<()>(c"context.set_value(42)").unwrap();
    }
    let res = vm.run_code::<()>(c"context.set_value(84)");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().into_runtime().unwrap().msg(),
        "[string \"context.set_value(84)\"]:1: Context is not available in this function."
    );
    assert_eq!(obj.value, 42);
    {
        let _obj = cell.bind(&mut obj);
        vm.run_code::<()>(c"assert(context.pop() == nil)").unwrap();
        vm.run_code::<()>(c"context.push(1)").unwrap();
        vm.run_code::<()>(c"context.push(2)").unwrap();
        vm.run_code::<()>(c"context.push(3)").unwrap();
    }
    assert_eq!(obj.value3.len(), 3);
    {
        let _obj = cell.bind(&mut obj);
        vm.run_code::<()>(c"assert(context.pop() == 3)").unwrap();
        vm.run_code::<()>(c"assert(context.pop() == 2)").unwrap();
        vm.run_code::<()>(c"assert(context.pop() == 1)").unwrap();
        vm.run_code::<()>(c"assert(context.pop() == nil)").unwrap();
        vm.run_code::<()>(c"assert(context.pop() == nil)").unwrap();
    }
    assert_eq!(obj.value3.len(), 0);
    assert_eq!(top, vm.top());
}

#[test]
fn test_vm_rust_closure_2() {
    let vm = RootVm::new();
    let top = vm.top();
    let (closure, _guard) =
        RClosure::from_rust_temporary(&vm, |val: f32| format!("this is a test: {}", val));
    vm.set_global(c"test", closure).unwrap();
    assert_eq!(top, vm.top());
    let s: &str = vm.run_code(c"return test(42.42)").unwrap();
    assert_eq!(s, "this is a test: 42.42");
}

#[test]
fn test_vm_rust_closure_3() {
    let vm = RootVm::new();
    let value = Cell::new(0);
    vm.scope(|vm| {
        let br = &value;
        let (fun, _guard) = RClosure::from_rust_temporary(vm, |()| br.get());
        vm.set_global(c"test", fun)?;
        let (fun2, _guard) = RClosure::from_rust_temporary(vm, |val: i32| br.set(val));
        vm.set_global(c"test2", fun2)?;
        vm.run_code::<()>(c"assert(test() == 0)")?;
        vm.run_code::<()>(c"test2(42)")
    })
    .unwrap();
    assert!(vm.run_code::<()>(c"test()").is_err());
    assert!(vm.run_code::<()>(c"test2(4242)").is_err());
    assert_eq!(value.get(), 42);
}

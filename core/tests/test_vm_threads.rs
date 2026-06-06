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

use bp3d_lua::vm::closure::rc::{Rc, Shared};
use bp3d_lua::vm::core::UnSendRootVm;
use bp3d_lua::vm::function::types::RFunction;
use bp3d_lua::vm::thread::core::{State, Yield};
use bp3d_lua::vm::thread::value::Thread;
use bp3d_lua::vm::value::types::Function;
use bp3d_lua::{decl_closure, decl_lib_func};
use std::cell::Cell;

decl_closure! {
    fn increment |val: Rc<Cell<u32>>| () -> () {
        val.set(val.get() + 1);
    }
}

#[test]
fn test_vm_threads_yield_lua() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            local value = coroutine.yield()
            if (value == 42) then
                increment()
            end
        end)
    ",
    )
    .unwrap();
    let thread: Thread = vm.get_global(c"CO").unwrap();
    assert_eq!(obj.get(), 0);
    assert_eq!(
        thread.as_thread().resume::<()>(()).unwrap().state,
        State::Suspended
    );
    assert_eq!(obj.get(), 1);
    assert_eq!(
        thread.as_thread().resume::<()>(42).unwrap().state,
        State::Finished
    );
    assert_eq!(obj.get(), 2);
    // A finished thread will fail to resume.
    assert!(thread.as_thread().resume::<()>(()).is_err());
    assert!(thread.as_thread().resume::<()>(true).is_err());
    assert!(thread.as_thread().resume::<()>(()).is_err());
}

decl_lib_func! {
    fn my_yield() -> Yield<()> {
        Yield(())
    }
}

decl_lib_func! {
    fn my_yield2(v: i32) -> Yield<i32> {
        Yield(v)
    }
}

#[test]
fn test_vm_threads_yield_rust_fail() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    vm.set_global(c"my_yield", RFunction::wrap(my_yield))
        .unwrap();
    let res = vm
        .run_code::<()>(c"my_yield()")
        .unwrap_err()
        .into_runtime()
        .unwrap();
    assert_eq!(
        res.msg(),
        "[string \"my_yield()\"]:1: attempt to yield a non-thread stack object"
    );
}

#[test]
fn test_vm_threads_yield_rust() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.set_global(c"my_yield", RFunction::wrap(my_yield))
        .unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            local value = my_yield()
            if (value == 42) then
                increment()
            end
        end)
    ",
    )
    .unwrap();
    let thread: Thread = vm.get_global(c"CO").unwrap();
    assert_eq!(obj.get(), 0);
    assert_eq!(
        thread.as_thread().resume::<()>(()).unwrap().state,
        State::Suspended
    );
    assert_eq!(obj.get(), 1);
    assert_eq!(
        thread.as_thread().resume::<()>(42).unwrap().state,
        State::Finished
    );
    assert_eq!(obj.get(), 2);
    // A finished thread will fail to resume.
    assert!(thread.as_thread().resume::<()>(()).is_err());
    assert!(thread.as_thread().resume::<()>(()).is_err());
    assert!(thread.as_thread().resume::<()>(()).is_err());
}

#[test]
fn test_vm_threads_with_yield_value_lua() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            coroutine.yield(1)
            increment()
            return 42
        end)
    ",
    )
    .unwrap();
    let thread: Thread = vm.get_global(c"CO").unwrap();
    assert_eq!(obj.get(), 0);
    assert_eq!(thread.as_thread().resume::<i32>(()).unwrap().data, 1);
    assert_eq!(thread.as_thread().resume::<i32>(()).unwrap().data, 42);
    assert_eq!(obj.get(), 2);
}

#[test]
fn test_vm_threads_with_yield_value_rust() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.set_global(c"my_yield", RFunction::wrap(my_yield2))
        .unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            my_yield(5)
            increment()
            return 42
        end)
    ",
    )
    .unwrap();
    let thread: Thread = vm.get_global(c"CO").unwrap();
    assert_eq!(obj.get(), 0);
    assert_eq!(thread.as_thread().resume::<i32>(()).unwrap().data, 5);
    assert_eq!(thread.as_thread().resume::<i32>(()).unwrap().data, 42);
    assert_eq!(obj.get(), 2);
}

#[test]
fn test_vm_threads_with_yield_value_unsafe() {
    let vm = UnSendRootVm::new();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            coroutine.yield(\"test\")
            increment()
            coroutine.yield(\"test2\")
            increment()
            collectgarbage()
            return \"test3\"
        end)
    ",
    )
    .unwrap();
    let thread: Thread = vm.get_global(c"CO").unwrap();
    assert_eq!(obj.get(), 0);
    let s: String = thread.as_thread().resume(()).unwrap().data;
    let s2: String = thread.as_thread().resume(()).unwrap().data;
    let s3: String = thread.as_thread().resume(()).unwrap().data;
    vm.run_code::<()>(c"CO = nil; collectgarbage()").unwrap();
    vm.run_code::<()>(
        c"
        CO = coroutine.create(function()
            increment()
            coroutine.yield(\"test\")
            increment()
            coroutine.yield(\"test2\")
            increment()
            collectgarbage()
            return \"test3\"
        end)
    ",
    )
    .unwrap();
    assert_eq!(s, "test");
    assert_eq!(s2, "test2");
    assert_eq!(s3, "test3");
    assert_eq!(obj.get(), 3);
}

#[test]
fn test_vm_threads_set_function() {
    let vm = UnSendRootVm::new();
    let top = vm.top();
    assert!(vm.as_thread().is_none());
    let obj = Shared::new(Cell::new(0));
    vm.set_global(c"increment", increment(Rc::from_rust(&vm, obj.clone())))
        .unwrap();
    vm.run_code::<()>(c"function ThreadMain() increment() coroutine.yield() increment() end")
        .unwrap();
    let main_fn: Function = vm.get_global(c"ThreadMain").unwrap();
    let thread = Thread::new(&vm);
    thread.set_function(main_fn).unwrap();
    assert_eq!(
        thread.as_thread().resume::<()>(()).unwrap().state,
        State::Suspended
    );
    assert_eq!(
        thread.as_thread().resume::<()>(()).unwrap().state,
        State::Finished
    );
    assert_eq!(obj.get(), 2);
    assert_eq!(vm.top() - top, 2)
}

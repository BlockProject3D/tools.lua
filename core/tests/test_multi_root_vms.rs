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

#![cfg(feature = "send")]

use bp3d_lua::vm::core::destructor::Pool;
use bp3d_lua::vm::registry::core::Key;
use bp3d_lua::vm::RootVm;
use bp3d_lua::vm::value::types::Function;
use bp3d_lua::vm::registry::types;

#[test]
fn test_multi_root_vms_basic() {
    let root1 = RootVm::new();
    let root2 = RootVm::new();
    let handle1 = std::thread::spawn(move || {
        root1.run_code::<()>(c"function Test() end").unwrap();
        let glb: Function = root1.get_global("Test").unwrap();
        glb.call::<()>(()).unwrap();
        (Key::<types::Function>::new(glb), root1)
    });
    let handle2 = std::thread::spawn(move || {
        root2.run_code::<()>(c"function Test2() end").unwrap();
        let glb: Option<Function> = root2.get_global("Test").unwrap();
        assert!(glb.is_none());
        let glb: Function = root2.get_global("Test2").unwrap();
        glb.call::<()>(()).unwrap();
        (Key::<types::Function>::new(glb), root2)
    });
    let (key1, root1) = handle1.join().unwrap();
    let (key2, root2) = handle2.join().unwrap();
    let fn1 = key1.push(&root1);
    let fn2 = key2.push(&root2);
    fn1.call::<()>(()).unwrap();
    fn2.call::<()>(()).unwrap();
}

#[test]
#[should_panic]
fn test_multi_root_vms_panic_1() {
    let root1 = RootVm::new();
    let root2 = RootVm::new();
    let handle1 = std::thread::spawn(move || {
        root1.run_code::<()>(c"function Test() end").unwrap();
        let glb: Function = root1.get_global("Test").unwrap();
        (Key::<types::Function>::new(glb), root1)
    });
    let handle2 = std::thread::spawn(move || {
        root2.run_code::<()>(c"function Test2() end").unwrap();
        let glb: Function = root2.get_global("Test2").unwrap();
        (Key::<types::Function>::new(glb), root2)
    });
    let (key1, _) = handle1.join().unwrap();
    let (_, root2) = handle2.join().unwrap();
    key1.push(&root2);
}

#[test]
#[should_panic]
fn test_multi_root_vms_panic_2() {
    let root1 = RootVm::new();
    let root2 = RootVm::new();
    let handle1 = std::thread::spawn(move || {
        root1.run_code::<()>(c"function Test() end").unwrap();
        let glb: Function = root1.get_global("Test").unwrap();
        (Key::<types::Function>::new(glb), root1)
    });
    let handle2 = std::thread::spawn(move || {
        root2.run_code::<()>(c"function Test2() end").unwrap();
        let glb: Function = root2.get_global("Test2").unwrap();
        (Key::<types::Function>::new(glb), root2)
    });
    let (_, root1) = handle1.join().unwrap();
    let (key2, _) = handle2.join().unwrap();
    key2.push(&root1);
}

#[test]
#[should_panic]
fn test_multi_root_vms_panic_3() {
    let root1 = RootVm::new();
    let handle1 = std::thread::spawn(move || {
        root1.run_code::<()>(c"function Test() end").unwrap();
        let glb: Function = root1.get_global("Test").unwrap();
        Key::<types::Function>::new(glb)
    });
    let key = handle1.join().unwrap();
    let root2 = RootVm::new();
    key.push(&root2);
}

#[test]
#[should_panic]
fn test_multi_root_vms_panic_4() {
    let root1 = RootVm::new();
    let handle1 = std::thread::spawn(move || {
        root1.run_code::<()>(c"function Test() end").unwrap();
        let glb: Function = root1.get_global("Test").unwrap();
        Key::<types::Function>::new(glb)
    });
    let key = handle1.join().unwrap();
    let root2 = RootVm::new();
    key.delete(&root2);
}

#[test]
#[should_panic]
fn test_multi_root_vms_not_send_destructor() {
    let root1 = RootVm::new();
    Pool::attach(&root1, Box::new(()));
}

#[test]
fn test_multi_root_vms_send_destructor() {
    let root1 = RootVm::new();
    Pool::attach_send(&root1, Box::new(()));
}

/*#[test]
fn test_multi_root_vms_not_send_build_error() {
    todo!()
}*/
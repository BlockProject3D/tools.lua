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

use bp3d_lua::vm::core::jit::{JitOptions, OptLevel};
use bp3d_lua::vm::RootVm;

#[test]
fn test_vm_get_jit_options() {
    let vm = RootVm::new();
    let jit = JitOptions::get(&vm);
    let optimizations = jit.opts();
    let cpu = jit.cpu();
    let is_on = jit.is_enabled();
    assert_eq!(is_on, true);
    assert_eq!(jit.opt_level(), OptLevel::default());
    println!("{} {}", cpu, optimizations);
}

#[test]
fn test_vm_set_jit_options() {
    let mut vm = RootVm::new();
    let mut jit = JitOptions::get(&vm);
    assert_eq!(jit.opt_level(), OptLevel::default());
    jit.set_opt_level(OptLevel::O0);
    assert_eq!(jit.opt_level(), OptLevel::O0);
    jit.apply(&mut vm);
    let jit = JitOptions::get(&vm);
    assert_eq!(jit.opt_level(), OptLevel::O0);
}

#[test]
fn test_vm_disable_jit() {
    let mut vm = RootVm::new();
    let mut jit = JitOptions::get(&vm);
    assert!(jit.is_enabled());
    jit.disable();
    jit.apply(&mut vm);
    let jit = JitOptions::get(&vm);
    assert!(!jit.is_enabled());
}

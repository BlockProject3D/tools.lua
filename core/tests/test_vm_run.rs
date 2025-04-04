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

use std::fmt::Write;
use bp3d_lua::ffi::lua::{State, ThreadStatus};
use bp3d_lua::vm::core::load::{load_custom, Code, Script};
use bp3d_lua::vm::core::Load;
use bp3d_lua::vm::core::util::ChunkNameBuilder;
use bp3d_lua::vm::{RootVm, Vm};

struct BrokenReader;

impl bp3d_lua::vm::core::load::Custom for BrokenReader {
    type Error = bp3d_lua::vm::error::Error;

    fn read_data(&mut self) -> Result<&[u8], Self::Error> {
        Err(bp3d_lua::vm::error::Error::Error)
    }
}

impl Load for BrokenReader {
    fn load(self, l: State) -> ThreadStatus {
        let mut builder = ChunkNameBuilder::new();
        let _ = write!(&mut builder, "broken");
        unsafe { load_custom(l, builder.build(), BrokenReader) }
    }
}

fn run_assert_err(vm: &Vm, obj: impl Load, err_msg: &str) {
    let res = vm.run::<()>(obj);
    assert!(res.is_err());
    let err = res.unwrap_err().into_runtime().unwrap();
    assert_eq!(err.msg(), err_msg);
}

#[test]
fn test_vm_run() {
    let vm = RootVm::new();
    let top = vm.top();
    run_assert_err(&vm, Code::new("test", b"return 1 + b"), "test:1: attempt to perform arithmetic on global 'b' (a nil value)");
    run_assert_err(&vm, c"return 1 + b", "[string \"return 1 + b\"]:1: attempt to perform arithmetic on global 'b' (a nil value)");
    run_assert_err(&vm, Code::new("this is an amazingly long text which should get truncated我", b"return 1 + b"), "this is an amazingly long text which should get truncated:1: attempt to perform arithmetic on global 'b' (a nil value)");
    let err = vm.run::<()>(BrokenReader).unwrap_err();
    assert_eq!(err.to_string(), "loader error: rust error: error in error handler");
    run_assert_err(&vm, Script::from_path("./tests/lua/basic.lua").unwrap(), "basic.lua:2: nope");
    let err = vm.run::<()>(Script::from_path("./tests/lua/broken.lua").unwrap()).unwrap_err();
    assert_eq!(err.to_string(), "syntax error: broken.lua:2: '(' expected near 'end'");
    vm.run::<()>(Script::from_path("./tests/lua/class.lua").unwrap()).unwrap();
    assert_eq!(vm.top(), top);
}

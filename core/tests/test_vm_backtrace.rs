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
use bp3d_lua::vm::function::types::RFunction;
use bp3d_lua::vm::value::function::Function;
use bp3d_lua::vm::RootVm;
use bp3d_util::simple_error;

simple_error! {
    pub Error {
        Useless => "useless function called"
    }
}

decl_lib_func! {
    fn error_func() -> Result<(), Error> {
        Err(Error::Useless)
    }
}

#[test]
fn test_vm_backtrace() {
    let vm = RootVm::new();
    let top = vm.top();
    vm.set_global(c"error_func", RFunction::wrap(error_func))
        .unwrap();
    vm.run_code::<()>(
        c"
        local function raise()
            error_func()
        end

        local function a()
            raise()
        end

        function main()
            a()
        end
    ",
    )
    .unwrap();
    let func: Function = vm.get_global(c"main").unwrap();
    let err = func.call::<()>(()).unwrap_err().into_runtime().unwrap();
    assert_eq!(err.msg(), "rust error: useless function called");
    assert_eq!(err.backtrace(), "rust error: useless function called\nstack traceback:\n\t[C]: in function 'error_func'\n\t[string \"...\"]:3: in function 'raise'\n\t[string \"...\"]:7: in function 'a'\n\t[string \"...\"]:11: in function <[string \"...\"]:10>");
    assert_eq!(vm.top(), top + 1);
}

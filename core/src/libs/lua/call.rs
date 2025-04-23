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

use crate::decl_lib_func;
use crate::libs::interface::Lib;
use crate::vm::error::Error;
use crate::vm::function::types::RFunction;
use crate::vm::namespace::Namespace;
use crate::vm::value::any::{AnyParam, UncheckedAnyReturn};
use crate::vm::value::function::Function;

decl_lib_func! {
    fn pcall(vm: &Vm, func: Function) -> UncheckedAnyReturn {
        let top = vm.top();
        true.into_param(vm);
        let ret = func.call::<AnyParam>(());
        let new_top = vm.top();
        match ret {
            Ok(_) => unsafe { UncheckedAnyReturn::new(vm, (new_top - top) as _) },
            Err(e) => {
                match e {
                    Error::Runtime(e) => unsafe { UncheckedAnyReturn::new(vm, (false, e.backtrace()).into_param(vm)) },
                    e => unsafe { UncheckedAnyReturn::new(vm, (false, e.to_string()).into_param(vm)) }
                }
            }
        }
    }
}

pub struct Call;

impl Lib for Call {
    const NAMESPACE: &'static str = "bp3d.lua";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([("pcall", RFunction::wrap(pcall))])
    }
}

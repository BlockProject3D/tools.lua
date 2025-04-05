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
use crate::vm::core::load::Code;
use crate::vm::function::IntoParam;
use crate::vm::function::types::RFunction;
use crate::vm::namespace::Namespace;
use crate::vm::value::any::{AnyParam, UncheckedAnyReturn};
use crate::vm::value::function::LuaFunction;
use crate::vm::Vm;

enum FunctionOrString<'a> {
    Func(LuaFunction<'a>),
    String(String)
}

unsafe impl IntoParam for FunctionOrString<'_> {
    fn into_param(self, vm: &Vm) -> u16 {
        match self {
            FunctionOrString::Func(v) => v.into_param(vm),
            FunctionOrString::String(v) => v.into_param(vm)
        }
    }
}

decl_lib_func! {
    fn dostring(vm: &Vm, s: &str, chunkname: Option<&str>) -> UncheckedAnyReturn {
        let top = vm.top();
        true.into_param(vm);
        let ret = match chunkname {
            None => vm.run_code::<AnyParam>(s),
            Some(name) => vm.run::<AnyParam>(Code::new(name, s.as_bytes()))
        };
        match ret {
            Ok(_) => {
                let new_top = vm.top();
                unsafe { UncheckedAnyReturn::new(vm, (new_top - top) as _) }
            },
            Err(e) => {
                let s = e.to_string();
                false.into_param(vm);
                s.into_param(vm);
                unsafe { UncheckedAnyReturn::new(vm, 2) }
            }
        }
    }
}

decl_lib_func! {
    fn loadstring<'a>(vm: &Vm, s: &str, chunkname: Option<&str>) -> (bool, FunctionOrString<'a>) {
        match chunkname {
            None => match vm.load_code(s) {
                Ok(v) => (true, FunctionOrString::Func(v)),
                Err(v) => (false, FunctionOrString::String(v.to_string()))
            },
            Some(name) => match vm.load(Code::new(name, s.as_bytes())) {
                Ok(v) => (true, FunctionOrString::Func(v)),
                Err(v) => (false, FunctionOrString::String(v.to_string()))
            }
        }
    }
}

pub fn register(vm: &Vm) -> crate::vm::Result<()> {
    let mut namespace = Namespace::new(vm, "bp3d.lua")?;
    namespace.add([
        ("dostring", RFunction::wrap(dostring)),
        ("loadstring", RFunction::wrap(loadstring))
    ])
}

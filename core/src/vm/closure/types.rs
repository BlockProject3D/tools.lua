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

use std::ffi::c_void;
use crate::ffi::lua::{lua_pushcclosure, CFunction, State};
use crate::vm::closure::{FromUpvalue, IntoUpvalue};
use crate::vm::value::IntoLua;
use crate::vm::{RootVm, Vm};
use crate::vm::function::{FromParam, IntoParam};

pub struct RClosure<T> {
    func: CFunction,
    upvalue: T
}

impl<T> RClosure<T> {
    /// Creates a new [RClosure].
    ///
    /// # Arguments
    ///
    /// * `func`: the [CFunction] to be associated with an upvalue.
    /// * `upvalue`: the upvalue to bind to the [CFunction].
    ///
    /// returns: RClosure<T>
    pub fn new(func: CFunction, upvalue: T) -> Self {
        Self { func, upvalue }
    }
}

impl<T: IntoUpvalue> IntoLua for RClosure<T> {
    fn into_lua(self, vm: &Vm) -> crate::vm::Result<u16> {
        let num = self.upvalue.into_upvalue(vm);
        unsafe { lua_pushcclosure(vm.as_ptr(), self.func, num as _) };
        Ok(1)
    }
}

impl RClosure<*const c_void> {
    pub fn from_rust<T, R, F: Fn(T) -> R + 'static>(root: &mut RootVm, fun: F) -> Self
        where for<'a> T: FromParam<'a>, R: IntoParam {
        let ptr = root.leak(Box::new(fun));
        extern "C-unwind" fn _cfunc<T, R, F: Fn(T) -> R>(l: State) -> i32
            where for<'a> T: FromParam<'a>, R: IntoParam {
            let vm = unsafe { Vm::from_raw(l) };
            let upvalue: *const F = unsafe { FromUpvalue::from_upvalue(&vm, 1) };
            let args: T = unsafe { FromParam::from_param(&vm, 1) };
            let res = unsafe { (*upvalue)(args) };
            res.into_param(&vm) as _
        }
        RClosure::new(_cfunc::<T, R, F>, ptr as *const _)
    }
}

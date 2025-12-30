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

use std::marker::PhantomData;
use crate::ffi::laux::luaL_error;
use crate::ffi::lua::{lua_newuserdata, lua_touserdata, State};
use crate::vm::closure::FromUpvalue;
use crate::vm::closure::types::RClosure;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::userdata::AnyUserData;
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;

pub struct Guard<'a, F> {
    ptr: *mut *const F,
    #[allow(unused)] // Hold the box until the guard should drop and delete the Box with it.
    bx: Box<F>,
    vm: PhantomData<&'a Vm>
}

impl<'a, F> Drop for Guard<'a, F> {
    fn drop(&mut self) {
        unsafe { *self.ptr = std::ptr::null() };
    }
}

impl<'a> RClosure<AnyUserData<'a>> {
    pub fn from_rust_temporary<'b, T, R, F: Fn(T) -> R + 'a>(vm: &'a Vm, fun: F) -> (RClosure<AnyUserData<'a>>, Guard<'a, F>)
    where
            T: FromParam<'b>,
            R: IntoParam
    {
        let bx = Box::new(fun);
        let rptr = &*bx as *const F;
        unsafe { lua_newuserdata(vm.as_ptr(), size_of::<*const F>()) };
        let ptr = unsafe { lua_touserdata(vm.as_ptr(), -1) } as *mut *const F;
        unsafe { *ptr = rptr };
        let value = unsafe { AnyUserData::from_raw(vm, vm.top()) };
        extern "C-unwind" fn _cfunc<'a, T, R, F: Fn(T) -> R>(l: State) -> i32
        where
                T: FromParam<'a>,
                R: IntoParam,
        {
            let vm = unsafe { Vm::from_raw(l) };
            let upvalue: RawPtr<*const F> = unsafe { FromUpvalue::from_upvalue(&vm, 1) };
            let args: T = unsafe { FromParam::from_param(std::mem::transmute(&vm), 1) };
            let ptr = unsafe { *upvalue.as_ptr() };
            if ptr.is_null() {
                unsafe { luaL_error(vm.as_ptr(), c"Attempt to call a dropped temporary rust closure".as_ptr()) };
                unsafe { std::hint::unreachable_unchecked() };
            }
            let res = unsafe { (*ptr)(args) };
            res.into_param(&vm) as _
        }
        (RClosure::new(_cfunc::<T, R, F>, value), Guard { ptr, bx, vm: PhantomData })
    }
}

impl RClosure<RawPtr<()>> {
    fn __from_rust<'a, T, R, F: Fn(T) -> R + 'static>(ptr: *mut F) -> Self
    where
            T: FromParam<'a>,
            R: IntoParam
    {
        extern "C-unwind" fn _cfunc<'a, T, R, F: Fn(T) -> R>(l: State) -> i32
        where
                T: FromParam<'a>,
                R: IntoParam,
        {
            let vm = unsafe { Vm::from_raw(l) };
            let upvalue: RawPtr<F> = unsafe { FromUpvalue::from_upvalue(&vm, 1) };
            let args: T = unsafe { FromParam::from_param(std::mem::transmute(&vm), 1) };
            let res = unsafe { (*upvalue.as_ptr())(args) };
            res.into_param(&vm) as _
        }
        RClosure::new(_cfunc::<T, R, F>, RawPtr::new(ptr as _))
    }

    #[cfg(feature = "send")]
    pub fn from_rust<'a, T, R, F: Fn(T) -> R + 'static>(vm: &Vm, fun: F) -> Self
    where
            T: FromParam<'a>,
            R: IntoParam,
            F: Send,
    {
        let ptr = crate::vm::core::destructor::Pool::attach_send(vm, Box::new(fun));
        Self::__from_rust(ptr)
    }

    #[cfg(not(feature = "send"))]
    pub fn from_rust<'a, T, R, F: Fn(T) -> R + 'static>(vm: &Vm, fun: F) -> Self
    where
            T: FromParam<'a>,
            R: IntoParam,
    {
        let ptr = crate::vm::core::destructor::Pool::attach(vm, Box::new(fun));
        Self::__from_rust(ptr)
    }
}

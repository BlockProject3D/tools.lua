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

use crate::ffi::lua::{lua_pushlightuserdata, lua_topointer, GLOBALSINDEX};
use crate::vm::closure::{FromUpvalue, IntoUpvalue};
use crate::vm::function::IntoParam;
use crate::vm::Vm;
use crate::vm::value::FromLua;

macro_rules! impl_from_upvalue_using_from_lua_unchecked {
    ($($t: ty),*) => {
        $(
            impl FromUpvalue<'_> for $t {
                #[inline(always)]
                unsafe fn from_upvalue(vm: &Vm, index: i32) -> Self {
                    <$t>::from_lua_unchecked(vm, GLOBALSINDEX - index)
                }
            }
        )*
    };
}

impl<'a> FromUpvalue<'a> for &'a str {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        FromLua::from_lua_unchecked(vm, GLOBALSINDEX - index)
    }
}

#[cfg(target_pointer_width = "64")]
impl_from_upvalue_using_from_lua_unchecked!(i64, u64);

impl_from_upvalue_using_from_lua_unchecked!(i8, u8, i16, u16, i32, u32, f32, f64, bool);

impl<T> FromUpvalue<'_> for *mut T {
    unsafe fn from_upvalue(vm: &Vm, index: i32) -> Self {
        lua_topointer(vm.as_ptr(), GLOBALSINDEX - index) as _
    }
}

impl<T> FromUpvalue<'_> for *const T {
    unsafe fn from_upvalue(vm: &'_ Vm, index: i32) -> Self {
        lua_topointer(vm.as_ptr(), GLOBALSINDEX - index) as _
    }
}

impl<T: IntoParam> IntoUpvalue for T {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.into_param(vm)
    }
}

impl<T> IntoUpvalue for *mut T {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlightuserdata(vm.as_ptr(), self as _) };
        1
    }
}

impl<T> IntoUpvalue for *const T {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlightuserdata(vm.as_ptr(), self as _) };
        1
    }
}

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

use crate::ffi::lua::GLOBALSINDEX;
use crate::vm::closure::{FromUpvalue, IntoUpvalue, Upvalue};
use crate::vm::function::IntoParam;
use crate::vm::value::types::RawPtr;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use std::ffi::OsStr;
use std::path::Path;

macro_rules! impl_from_upvalue_using_from_lua_unchecked {
    ($($t: ty),*) => {
        $(
            impl FromUpvalue<'_> for $t {
                #[inline(always)]
                unsafe fn from_upvalue(vm: &Vm, index: i32) -> Self {
                    <$t>::from_lua_unchecked(vm, GLOBALSINDEX - index)
                }
            }

            impl Upvalue for $t {
                type From<'a> = $t;
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

impl Upvalue for &str {
    type From<'a> = &'a str;
}

impl<'a> FromUpvalue<'a> for &'a [u8] {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        FromLua::from_lua_unchecked(vm, GLOBALSINDEX - index)
    }
}

impl Upvalue for &[u8] {
    type From<'a> = &'a [u8];
}

#[cfg(target_pointer_width = "64")]
impl_from_upvalue_using_from_lua_unchecked!(i64, u64);

impl_from_upvalue_using_from_lua_unchecked!(i8, u8, i16, u16, i32, u32, f32, f64, bool);

impl<T> FromUpvalue<'_> for RawPtr<T> {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &Vm, index: i32) -> Self {
        RawPtr::from_lua(vm, GLOBALSINDEX - index)
    }
}

impl<T: IntoParam + Upvalue> IntoUpvalue for T {
    #[inline(always)]
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.into_param(vm) as _
    }
}

impl<T> IntoUpvalue for RawPtr<T> {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.into_lua(vm);
        1
    }
}

impl<T> Upvalue for RawPtr<T> {
    type From<'a> = RawPtr<T>;
}

impl<'a> FromUpvalue<'a> for &'a OsStr {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        OsStr::from_encoded_bytes_unchecked(FromUpvalue::from_upvalue(vm, index))
    }
}

impl IntoUpvalue for &OsStr {
    #[inline(always)]
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.as_encoded_bytes().into_upvalue(vm)
    }
}

impl Upvalue for &OsStr {
    type From<'a> = &'a OsStr;
}

impl<'a> FromUpvalue<'a> for &'a Path {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        Path::new(OsStr::from_encoded_bytes_unchecked(
            FromUpvalue::from_upvalue(vm, index),
        ))
    }
}

impl IntoUpvalue for &Path {
    #[inline(always)]
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.as_os_str().into_upvalue(vm)
    }
}

impl Upvalue for &Path {
    type From<'a> = &'a Path;
}

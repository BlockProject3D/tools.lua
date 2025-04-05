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

use std::error::Error;
use std::slice;
use crate::ffi::laux::{luaL_checklstring, luaL_checkudata, luaL_setmetatable, luaL_testudata};
use crate::ffi::lua::{lua_newuserdata, lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_type, Integer, Number, Type};
use crate::ffi::ext::{lua_ext_fast_checknumber, lua_ext_fast_checkinteger};
use crate::util::SimpleDrop;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::userdata::UserData;
use crate::vm::util::{lua_rust_error, LuaType, TypeName};
use crate::vm::value::FromLua;
use crate::vm::Vm;

impl<'a, T: FromParam<'a> + SimpleDrop> FromParam<'a> for Option<T> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        let l = vm.as_ptr();
        let ty = lua_type(l, index);
        if ty == Type::Nil || ty == Type::None {
            None
        } else {
            Some(T::from_param(vm, index))
        }
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        let l = vm.as_ptr();
        let ty = unsafe { lua_type(l, index) };
        if ty == Type::Nil || ty == Type::None {
            None
        } else {
            Some(T::try_from_param(vm, index))
        }
    }
}

impl LuaType for &str { }

impl<'a> FromParam<'a> for &'a str {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        let mut len: usize = 0;
        let str = luaL_checklstring(vm.as_ptr(), index, &mut len as _);
        let slice = slice::from_raw_parts(str as *const u8, len);
        match std::str::from_utf8(slice){
            Ok(v) => v,
            Err(e) => {
                lua_rust_error(vm.as_ptr(), e);
            }
        }
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        FromLua::from_lua(vm, index).ok()
    }
}

impl LuaType for &[u8] {}

impl<'a> FromParam<'a> for &'a [u8] {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        let mut len: usize = 0;
        let str = luaL_checklstring(vm.as_ptr(), index, &mut len as _);
        let slice = slice::from_raw_parts(str as *const u8, len);
        slice
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        FromLua::from_lua(vm, index).ok()
    }
}

unsafe impl IntoParam for &str {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlstring(vm.as_ptr(), self.as_ptr() as _, self.len()); }
        1
    }
}

unsafe impl IntoParam for &[u8] {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlstring(vm.as_ptr(), self.as_ptr() as _, self.len()); }
        1
    }
}

unsafe impl IntoParam for String {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> u16 {
        (&*self).into_param(vm)
    }
}

macro_rules! impl_integer {
    ($($t: ty),*) => {
        $(
            unsafe impl SimpleDrop for $t {}

            impl LuaType for $t {
                fn lua_type() -> Vec<TypeName> {
                    vec![TypeName::Some(std::any::type_name::<Integer>())]
                }
            }

            impl FromParam<'_> for $t {
                #[inline(always)]
                unsafe fn from_param(vm: &Vm, index: i32) -> Self {
                    lua_ext_fast_checkinteger(vm.as_ptr(), index) as _
                }

                #[inline(always)]
                fn try_from_param(vm: &Vm, index: i32) -> Option<Self> {
                    FromLua::from_lua(vm, index).ok()
                }
            }

            unsafe impl IntoParam for $t {
                #[inline(always)]
                fn into_param(self, vm: &Vm) -> u16 {
                    unsafe {
                        lua_pushinteger(vm.as_ptr(), self as _);
                        1
                    }
                }
            }
        )*
    };
}

#[cfg(target_pointer_width = "64")]
impl_integer!(i64, u64);

impl_integer!(i8, u8, i16, u16, i32, u32);

macro_rules! impl_float {
    ($($t: ty),*) => {
        $(
            unsafe impl SimpleDrop for $t {}

            impl LuaType for $t {
                fn lua_type() -> Vec<TypeName> {
                    vec![TypeName::Some(std::any::type_name::<Number>())]
                }
            }

            impl FromParam<'_> for $t {
                #[inline(always)]
                unsafe fn from_param(vm: &Vm, index: i32) -> Self {
                    lua_ext_fast_checknumber(vm.as_ptr(), index) as _
                }

                #[inline(always)]
                fn try_from_param(vm: &Vm, index: i32) -> Option<Self> {
                    FromLua::from_lua(vm, index).ok()
                }
            }

            unsafe impl IntoParam for $t {
                #[inline(always)]
                fn into_param(self, vm: &Vm) -> u16 {
                    unsafe {
                        lua_pushnumber(vm.as_ptr(), self as _);
                        1
                    }
                }
            }
        )*
    };
}

impl_float!(f32, f64);

unsafe impl IntoParam for bool {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe { lua_pushboolean(vm.as_ptr(), if self { 1 } else { 0 }) };
        1
    }
}

unsafe impl<T: IntoParam, E: Error> IntoParam for Result<T, E> {
    fn into_param(self, vm: &Vm) -> u16 {
        match self {
            Ok(v) => v.into_param(vm),
            Err(e) => {
                unsafe {
                    lua_rust_error(vm.as_ptr(), e);
                }
            }
        }
    }
}

unsafe impl<T: IntoParam> IntoParam for Option<T> {
    fn into_param(self, vm: &Vm) -> u16 {
        match self {
            None => {
                unsafe {
                    lua_pushnil(vm.as_ptr());
                    1
                }
            }
            Some(v) => v.into_param(vm)
        }
    }
}

unsafe impl IntoParam for () {
    #[inline(always)]
    fn into_param(self, _: &Vm) -> u16 {
        0
    }
}

impl<T: UserData> LuaType for &T {
    fn lua_type() -> Vec<TypeName> {
        vec![TypeName::Some(unsafe { T::CLASS_NAME.to_str().unwrap_unchecked() })]
    }
}

impl<'a, T: UserData> FromParam<'a> for &'a T {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> &'a T {
        let obj_ptr = luaL_checkudata(vm.as_ptr(), index, T::CLASS_NAME.as_ptr()) as *const T;
        unsafe { &*obj_ptr }
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        let ptr = unsafe { luaL_testudata(vm.as_ptr(), index, T::CLASS_NAME.as_ptr()) } as *const T;
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*ptr })
        }
    }
}

unsafe impl<T: UserData> IntoParam for T {
    fn into_param(self, vm: &Vm) -> u16 {
        let userdata = unsafe { lua_newuserdata(vm.as_ptr(), size_of::<T>()) } as *mut T;
        unsafe { userdata.write(self) };
        unsafe { luaL_setmetatable(vm.as_ptr(), T::CLASS_NAME.as_ptr()) };
        1
    }
}

macro_rules! count_tts {
    () => {0};
    ($_head:tt $($tail:tt)*) => {1 + count_tts!($($tail)*)};
}

macro_rules! impl_into_param_tuple {
    ($($name: ident: $name2: tt),*) => {
        unsafe impl<$($name: IntoParam),*> IntoParam for ($($name),*) {
            fn into_param(self, vm: &Vm) -> u16 {
                $(
                    self.$name2.into_param(vm);
                )*
                count_tts!($($name)*)
            }
        }
    };
}

impl_into_param_tuple!(T: 0, T1: 1);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8);
impl_into_param_tuple!(T: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9);

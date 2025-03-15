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
use crate::ffi::laux::{luaL_checklstring, luaL_checkudata, luaL_setmetatable};
use crate::ffi::lua::{lua_newuserdata, lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_type, Integer, Number, Type};
use crate::ffi::ext::{lua_ext_fast_checknumber, lua_ext_fast_checkinteger};
use crate::util::SimpleDrop;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::userdata::UserData;
use crate::vm::util::{lua_rust_error, LuaType, TypeName};
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
}

unsafe impl SimpleDrop for &str {}

impl IntoParam for &str {
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe {
            lua_pushlstring(vm.as_ptr(), self.as_ptr() as _, self.len());
        }
        1
    }
}

impl IntoParam for String {
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
                unsafe fn from_param(vm: &Vm, index: i32) -> Self {
                    lua_ext_fast_checkinteger(vm.as_ptr(), index) as _
                }
            }

            impl IntoParam for $t {
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
                unsafe fn from_param(vm: &Vm, index: i32) -> Self {
                    lua_ext_fast_checknumber(vm.as_ptr(), index) as _
                }
            }

            impl IntoParam for $t {
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

impl IntoParam for bool {
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe { lua_pushboolean(vm.as_ptr(), if self { 1 } else { 0 }) };
        1
    }
}

impl<T: IntoParam, E: Error> IntoParam for Result<T, E> {
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

impl<T: IntoParam> IntoParam for Option<T> {
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

impl IntoParam for () {
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
    unsafe fn from_param(vm: &'a Vm, index: i32) -> &'a T {
        let obj_ptr = unsafe { luaL_checkudata(vm.as_ptr(), index, T::CLASS_NAME.as_ptr()) } as *const T;
        unsafe { &*obj_ptr }
    }
}

impl<T: UserData> IntoParam for T {
    fn into_param(self, vm: &Vm) -> u16 {
        let userdata = unsafe { lua_newuserdata(vm.as_ptr(), size_of::<T>()) } as *mut T;
        unsafe { userdata.write(self) };
        unsafe { luaL_setmetatable(vm.as_ptr(), T::CLASS_NAME.as_ptr()) };
        1
    }
}

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

use crate::check_single_type;
use crate::ffi::laux::luaL_testudata;
use crate::ffi::lua::{lua_tolstring, lua_type, lua_tointeger, lua_tonumber, Type, lua_toboolean, lua_touserdata};
use crate::vm::function::IntoParam;
use crate::vm::Vm;
use crate::vm::error::{Error, TypeError};
use crate::vm::userdata::UserDataImmutable;
use crate::vm::value::{FromLua, IntoLua};

impl<'a> FromLua<'a> for &'a str {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut len: usize = 0;
        let s = lua_tolstring(vm.as_ptr(), index, &mut len as _);
        let slice = std::slice::from_raw_parts(s as _, len);
        std::str::from_utf8_unchecked(slice)
    }

    fn from_lua(vm: &Vm, index: i32) -> crate::vm::Result<Self> {
        let l = vm.as_ptr();
        unsafe {
            let ty = lua_type(l, index);
            match ty {
                Type::String => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(l, index, &mut len as _);
                    let slice = std::slice::from_raw_parts(s as _, len);
                    std::str::from_utf8(slice).map_err(Error::InvalidUtf8)
                },
                _ => Err(Error::Type(TypeError {
                    expected: Type::String,
                    actual: ty
                }))
            }
        }
    }
}

macro_rules! impl_from_lua {
    ($t: ty, $expected: ident, $func: ident, $($ret: tt)*) => {
        impl FromLua<'_> for $t {
            #[inline(always)]
            unsafe fn from_lua_unchecked(vm: &Vm, index: i32) -> Self {
                $func(vm.as_ptr(), index) $($ret)*
            }

            fn from_lua(vm: &Vm, index: i32) -> crate::vm::Result<Self> {
                unsafe {
                    check_single_type!(Type::$expected => (vm, index) { $func(vm.as_ptr(), index) $($ret)* })
                }
            }
        }
    };
}

#[cfg(target_pointer_width = "64")]
impl_from_lua!(i64, Number, lua_tointeger, as _);

#[cfg(target_pointer_width = "64")]
impl_from_lua!(u64, Number, lua_tointeger, as _);

impl_from_lua!(i8, Number, lua_tointeger, as _);
impl_from_lua!(u8, Number, lua_tointeger, as _);
impl_from_lua!(i16, Number, lua_tointeger, as _);
impl_from_lua!(u16, Number, lua_tointeger, as _);
impl_from_lua!(i32, Number, lua_tointeger, as _);
impl_from_lua!(u32, Number, lua_tointeger, as _);

impl_from_lua!(f32, Number, lua_tonumber, as _);
impl_from_lua!(f64, Number, lua_tonumber, as _);

impl_from_lua!(bool, Boolean, lua_toboolean, == 1);

impl<T: IntoParam> IntoLua for T {
    fn into_lua(self, vm: &Vm) -> Result<u16, Error> {
        Ok(self.into_param(vm))
    }
}

impl FromLua<'_> for () {
    unsafe fn from_lua_unchecked(_: &'_ Vm, _: i32) -> Self {
        ()
    }

    fn from_lua(_vm: &Vm, _: i32) -> crate::vm::Result<()> {
        Ok(())
    }

    fn num_values() -> u16 {
        0
    }
}

impl<'a, T: UserDataImmutable> FromLua<'a> for &'a T {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        &*(lua_touserdata(vm.as_ptr(), index) as *const T)
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let this_ptr = unsafe { luaL_testudata(vm.as_ptr(), index, T::CLASS_NAME.as_ptr()) } as *const T;
        if this_ptr.is_null() {
            return Err(Error::Type(TypeError {
                expected: Type::Userdata,
                actual: unsafe { lua_type(vm.as_ptr(), index) },
            }));
        }
        Ok(unsafe { &*this_ptr })
    }
}

macro_rules! count_tts {
    () => {0};
    ($_head:tt $($tail:tt)*) => {1 + count_tts!($($tail)*)};
}

macro_rules! impl_tuple {
    ($($name: ident: $name2: ident),*) => {
        impl<'a, $($name: FromLua<'a>),*> FromLua<'a> for ($($name),*) {
            fn num_values() -> u16 {
                count_tts!($($name),*)
            }

            unsafe fn from_lua_unchecked(vm: &'a Vm, mut index: i32) -> Self {
                $(
                    let $name2: $name = FromLua::from_lua_unchecked(vm, index);
                    index += 1;
                )*
                ($($name2),*)
            }

            fn from_lua(vm: &'a Vm, mut index: i32) -> crate::vm::Result<($($name),*)> {
                $(
                    let $name2: $name = FromLua::from_lua(vm, index)?;
                    index += 1;
                )*
                Ok(($($name2),*))
            }
        }
    };
}

impl_tuple!(T: t, T1: t1);
impl_tuple!(T: t, T1: t1, T2: t2);
impl_tuple!(T: t, T1: t1, T2: t2, T3: t3);

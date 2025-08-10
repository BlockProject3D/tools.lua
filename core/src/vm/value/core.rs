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

use std::borrow::Cow;
use crate::ffi::laux::{luaL_setmetatable, luaL_testudata};
use crate::ffi::lua::{lua_newuserdata, lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_settop, lua_toboolean, lua_tointeger, lua_tointegerx, lua_tolstring, lua_tonumber, lua_tonumberx, lua_touserdata, lua_type, Type};
use crate::vm::error::{Error, TypeError};
use crate::vm::userdata::{UserData, UserDataImmutable};
use crate::vm::value::util::check_type_equals;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::types::{Boolean, Integer, Number};
use crate::vm::Vm;

impl<'a> FromLua<'a> for &'a str {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut len: usize = 0;
        let s = lua_tolstring(vm.as_ptr(), index, &mut len as _);
        let slice = std::slice::from_raw_parts(s as _, len);
        std::str::from_utf8_unchecked(slice)
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let l = vm.as_ptr();
        unsafe {
            let ty = lua_type(l, index);
            match ty {
                Type::String => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(l, index, &mut len as _);
                    let slice = std::slice::from_raw_parts(s as _, len);
                    std::str::from_utf8(slice).map_err(|e| Error::InvalidUtf8(e.into()))
                }
                _ => Err(Error::Type(TypeError {
                    expected: Type::String,
                    actual: ty,
                })),
            }
        }
    }
}

impl<'a> FromLua<'a> for &'a [u8] {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut len: usize = 0;
        let str = lua_tolstring(vm.as_ptr(), index, &mut len as _);
        let slice = std::slice::from_raw_parts(str as *const u8, len);
        slice
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let l = vm.as_ptr();
        unsafe {
            let ty = lua_type(l, index);
            match ty {
                Type::String => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(l, index, &mut len as _);
                    let slice = std::slice::from_raw_parts(s as *const u8, len);
                    Ok(slice)
                }
                _ => Err(Error::Type(TypeError {
                    expected: Type::String,
                    actual: ty,
                })),
            }
        }
    }
}

impl FromLua<'_> for String {
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        let s: &str = FromLua::from_lua_unchecked(vm, index);
        s.into()
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        let s: &str = FromLua::from_lua(vm, index)?;
        Ok(s.into())
    }
}

impl FromLua<'_> for Box<[u8]> {
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        let bytes: &[u8] = FromLua::from_lua_unchecked(vm, index);
        bytes.into()
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        let bytes: &[u8] = FromLua::from_lua(vm, index)?;
        Ok(bytes.into())
    }
}

macro_rules! impl_from_lua {
    ($t: ty, $expected: ident, $func: ident, $push_func: ident, $($ret: tt)*) => {
        impl FromLua<'_> for $t {
            #[inline(always)]
            unsafe fn from_lua_unchecked(vm: &Vm, index: i32) -> Self {
                $func(vm.as_ptr(), index) $($ret)*
            }

            fn from_lua(vm: &Vm, index: i32) -> crate::vm::Result<Self> {
                check_type_equals(vm, index, Type::$expected)?;
                Ok(unsafe { $func(vm.as_ptr(), index) $($ret)* })
            }
        }

        unsafe impl IntoLua for $t {
            #[inline(always)]
            fn into_lua(self, vm: &Vm) -> u16 {
                unsafe {
                    $push_func(vm.as_ptr(), self as _);
                    1
                }
            }
        }
    };
}

#[cfg(target_pointer_width = "64")]
impl_from_lua!(i64, Number, lua_tointeger, lua_pushinteger, as _);

#[cfg(target_pointer_width = "64")]
impl_from_lua!(u64, Number, lua_tointeger, lua_pushinteger, as _);

impl_from_lua!(i8, Number, lua_tointeger, lua_pushinteger, as _);
impl_from_lua!(u8, Number, lua_tointeger, lua_pushinteger, as _);
impl_from_lua!(i16, Number, lua_tointeger, lua_pushinteger, as _);
impl_from_lua!(u16, Number, lua_tointeger, lua_pushinteger, as _);
impl_from_lua!(i32, Number, lua_tointeger, lua_pushinteger, as _);
impl_from_lua!(u32, Number, lua_tointeger, lua_pushinteger, as _);

impl_from_lua!(f32, Number, lua_tonumber, lua_pushnumber, as _);
impl_from_lua!(f64, Number, lua_tonumber, lua_pushnumber, as _);

impl_from_lua!(bool, Boolean, lua_toboolean, lua_pushboolean, == 1);

impl FromLua<'_> for () {
    #[inline(always)]
    unsafe fn from_lua_unchecked(_: &'_ Vm, _: i32) -> Self {}

    #[inline(always)]
    fn from_lua(_vm: &Vm, _: i32) -> crate::vm::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn num_values() -> i16 {
        0
    }
}

impl<'a, T: UserDataImmutable> FromLua<'a> for &'a T {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        &*(lua_touserdata(vm.as_ptr(), index) as *const T)
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let this_ptr =
            unsafe { luaL_testudata(vm.as_ptr(), index, T::CLASS_NAME.as_ptr()) } as *const T;
        if this_ptr.is_null() {
            return Err(Error::Type(TypeError {
                expected: Type::Userdata,
                actual: unsafe { lua_type(vm.as_ptr(), index) },
            }));
        }
        Ok(unsafe { &*this_ptr })
    }
}

macro_rules! impl_from_lua_tuple {
    ($($name: ident: $name2: ident ($name3: tt)),*) => {
        impl<'a, $($name: FromLua<'a>),*> FromLua<'a> for ($($name),*) {
            #[inline(always)]
            fn num_values() -> i16 {
                $($name::num_values()+)*
                0
            }

            unsafe fn from_lua_unchecked(vm: &'a Vm, mut index: i32) -> Self {
                impl_from_lua_tuple!(_from_lua_unchecked vm, index, $($name2: $name),*);
                ($($name2),*)
            }

            fn from_lua(vm: &'a Vm, mut index: i32) -> crate::vm::Result<($($name),*)> {
                impl_from_lua_tuple!(_from_lua vm, index, $($name2: $name),*);
                Ok(($($name2),*))
            }
        }

        unsafe impl<$($name: IntoLua),*> IntoLua for ($($name),*) {
            fn into_lua(self, vm: &Vm) -> u16 {
                $(
                    self.$name3.into_lua(vm) +
                )*
                0
            }
        }
    };

    (_from_lua_unchecked $vm: ident, $index: ident, $name2: ident: $name: ident) => {
        let $name2: $name = FromLua::from_lua_unchecked($vm, $index);
    };

    (_from_lua_unchecked $vm: ident, $index: ident, $name2: ident: $name: ident, $($name3: ident: $name4: ident),*) => {
        let $name2: $name = FromLua::from_lua_unchecked($vm, $index);
        $index += 1;
        impl_from_lua_tuple!(_from_lua_unchecked $vm, $index, $($name3: $name4),*);
    };

    (_from_lua $vm: ident, $index: ident, $name2: ident: $name: ident) => {
        let $name2: $name = FromLua::from_lua($vm, $index)?;
    };

    (_from_lua $vm: ident, $index: ident, $name2: ident: $name: ident, $($name3: ident: $name4: ident),*) => {
        let $name2: $name = FromLua::from_lua($vm, $index)?;
        $index += 1;
        impl_from_lua_tuple!(_from_lua $vm, $index, $($name3: $name4),*);
    };
}

impl_from_lua_tuple!(T: t (0), T1: t1 (1));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4), T5: t5 (5));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4), T5: t5 (5), T6: t6 (6));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4), T5: t5 (5), T6: t6 (6), T7: t7 (7));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4), T5: t5 (5), T6: t6 (6), T7: t7 (7), T8: t8 (8));
impl_from_lua_tuple!(T: t (0), T1: t1 (1), T2: t2 (2), T3: t3 (3), T4: t4 (4), T5: t5 (5), T6: t6 (6), T7: t7 (7), T8: t8 (8), T9: t9 (9));

impl<'a, T: FromLua<'a>> FromLua<'a> for Option<T> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        if ty == Type::Nil {
            // Clear the nil value at the top of the stack.
            if index == -1 {
                unsafe { lua_settop(vm.as_ptr(), -2) };
            }
            None
        } else {
            Some(FromLua::from_lua_unchecked(vm, index))
        }
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        if ty == Type::Nil {
            // Clear the nil value at the top of the stack.
            if index == -1 {
                unsafe { lua_settop(vm.as_ptr(), -2) };
            }
            Ok(None)
        } else {
            Ok(Some(FromLua::from_lua(vm, index)?))
        }
    }
}

unsafe impl<T: UserData> IntoLua for T {
    fn into_lua(self, vm: &Vm) -> u16 {
        let userdata = unsafe { lua_newuserdata(vm.as_ptr(), size_of::<T>()) } as *mut T;
        unsafe { userdata.write(self) };
        unsafe { luaL_setmetatable(vm.as_ptr(), T::CLASS_NAME.as_ptr()) };
        1
    }
}

unsafe impl IntoLua for () {
    #[inline(always)]
    fn into_lua(self, _: &Vm) -> u16 {
        0
    }
}

unsafe impl<T: IntoLua> IntoLua for Option<T> {
    fn into_lua(self, vm: &Vm) -> u16 {
        match self {
            None => unsafe {
                lua_pushnil(vm.as_ptr());
                1
            },
            Some(v) => v.into_lua(vm),
        }
    }
}

unsafe impl IntoLua for &str {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        self.as_bytes().into_lua(vm)
    }
}

unsafe impl IntoLua for &[u8] {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlstring(vm.as_ptr(), self.as_ptr() as _, self.len()) };
        1
    }
}

unsafe impl IntoLua for String {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        (&*self).into_lua(vm)
    }
}

unsafe impl IntoLua for Box<[u8]> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        self.as_ref().into_lua(vm)
    }
}

unsafe impl<'a, T: IntoLua + Clone> IntoLua for Cow<'a, T>
where
    &'a T: IntoLua,
{
    fn into_lua(self, vm: &Vm) -> u16 {
        match self {
            Cow::Borrowed(v) => v.into_lua(vm),
            Cow::Owned(v) => v.into_lua(vm),
        }
    }
}

impl FromLua<'_> for Integer {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        Integer(lua_tointeger(vm.as_ptr(), index))
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        let mut ok = 0;
        let num = unsafe { lua_tointegerx(vm.as_ptr(), index, &mut ok) };
        if ok != 1 {
            Err(TypeError::from_stack(Type::Number, vm, index))
        } else {
            Ok(Integer(num))
        }
    }
}

impl FromLua<'_> for Number {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        Number(lua_tonumber(vm.as_ptr(), index))
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        let mut ok = 0;
        let num = unsafe { lua_tonumberx(vm.as_ptr(), index, &mut ok) };
        if ok != 1 {
            Err(TypeError::from_stack(Type::Number, vm, index))
        } else {
            Ok(Number(num))
        }
    }
}

impl FromLua<'_> for Boolean {
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        Boolean(unsafe { lua_toboolean(vm.as_ptr(), index) == 1 })
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        Ok(Boolean(unsafe { lua_toboolean(vm.as_ptr(), index) == 1 }))
    }
}

unsafe impl IntoLua for Number {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushnumber(vm.as_ptr(), self.0) }
        1
    }
}

unsafe impl IntoLua for Integer {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushinteger(vm.as_ptr(), self.0) }
        1
    }
}

unsafe impl IntoLua for Boolean {
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushboolean(vm.as_ptr(), if self.0 { 1 } else { 0 }) }
        1
    }
}

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

use crate::ffi::lua::{lua_tolstring, lua_type, lua_tointeger, lua_tonumber, Type, lua_toboolean, CFunction, lua_pushcclosure};
use crate::vm::function::IntoParam;
use crate::vm::{LuaState, Stack};
use crate::vm::error::{Error, TypeError};

pub trait FromLua<'a>: Sized {
    /// Attempt to read the value at the specified index in the given [LuaState].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [LuaState] to read from.
    /// * `index`: the index at which to try reading the value from.
    ///
    /// returns: Result<Self, Error>
    fn from_lua(vm: &'a LuaState, index: i32) -> crate::vm::Result<Self>;

    /// Returns the number of values to be expected on the lua stack, after reading this value.
    fn num_values() -> u16 {
        1
    }
}

pub trait IntoLua: Sized {
    /// Attempt to push self onto the top of the stack in the given [LuaState].
    ///
    /// Returns the number values pushed into the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [LuaState] to push into.
    ///
    /// returns: Result<Self, Error>
    fn into_lua(self, vm: &LuaState) -> crate::vm::Result<u16>;
}

impl<'a> FromLua<'a> for &'a str {
    fn from_lua(vm: &LuaState, index: i32) -> crate::vm::Result<Self> {
        let l = **vm;
        unsafe {
            let ty = lua_type(l, index);
            match ty {
                Type::String => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(l, index, &mut len as _);
                    let slice = std::slice::from_raw_parts(s as _, len);
                    std::str::from_utf8(slice).map_err(Error::InvalidUtf8)
                },
                _ => Err(Error::TypeError(TypeError {
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
            fn from_lua(vm: &LuaState, index: i32) -> crate::vm::Result<Self> {
                let l = **vm;
                unsafe {
                    let ty = lua_type(l, index);
                    match ty {
                        Type::$expected => Ok($func(l, index) $($ret)*),
                        _ => Err(Error::TypeError(TypeError {
                            expected: Type::$expected,
                            actual: ty
                        }))
                    }
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
    fn into_lua(self, vm: &LuaState) -> Result<u16, Error> {
        let stack = unsafe { Stack::wrap(**vm, 0) };
        Ok(self.into_param(&stack))
    }
}

pub struct RFunction(pub CFunction);

impl IntoLua for RFunction {
    fn into_lua(self, vm: &LuaState) -> crate::vm::Result<u16> {
        let l = **vm;
        unsafe {
            lua_pushcclosure(l, self.0, 0);
        }
        Ok(1)
    }
}

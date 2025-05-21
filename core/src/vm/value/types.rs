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

use crate::ffi::laux::{luaL_checkinteger, luaL_checknumber};
use crate::ffi::lua::{lua_isnumber, lua_tointeger, lua_tonumber, lua_type, RawInteger, RawNumber, Type};
use crate::util::core::SimpleDrop;
use crate::vm::error::{Error, TypeError};
use crate::vm::function::FromParam;
use crate::vm::util::LuaType;
use crate::vm::value::FromLua;
use crate::vm::Vm;

pub use super::function::Function;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Number(pub RawNumber);

unsafe impl SimpleDrop for Number {}

impl LuaType for Number {}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Eq, Ord, Hash)]
pub struct Integer(pub RawInteger);

unsafe impl SimpleDrop for Integer {}

impl LuaType for Integer {}

impl<'a> FromParam<'a> for Number {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        Self(luaL_checknumber(vm.as_ptr(), index))
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        let l = vm.as_ptr();
        unsafe {
            if lua_isnumber(l, index) == 1 {
                Some(Self(lua_tonumber(l, index)))
            } else {
                None
            }
        }
    }
}

impl<'a> FromLua<'a> for Number {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self(lua_tonumber(vm.as_ptr(), index))
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let l = vm.as_ptr();
        unsafe {
            if lua_isnumber(l, index) != 1 {
                return Err(Error::Type(TypeError {
                    expected: Type::Number,
                    actual: lua_type(l, index),
                }));
            }
            Ok(Self(lua_tonumber(l, index)))
        }
    }
}

impl<'a> FromParam<'a> for Integer {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        Self(luaL_checkinteger(vm.as_ptr(), index))
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        let l = vm.as_ptr();
        unsafe {
            if lua_isnumber(l, index) == 1 {
                Some(Self(lua_tointeger(l, index)))
            } else {
                None
            }
        }
    }
}

impl<'a> FromLua<'a> for Integer {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self(lua_tointeger(vm.as_ptr(), index))
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let l = vm.as_ptr();
        unsafe {
            if lua_isnumber(l, index) != 1 {
                return Err(Error::Type(TypeError {
                    expected: Type::Number,
                    actual: lua_type(l, index),
                }));
            }
            Ok(Self(lua_tointeger(l, index)))
        }
    }
}

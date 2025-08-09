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

use crate::ffi::lua::{lua_pushnil, lua_toboolean, lua_tonumber, lua_type, Type};
use crate::util::core::SimpleDrop;
use crate::vm::error::{Error, TypeError};
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::table::Table;
use crate::vm::thread::value::Thread as Thread;
use crate::vm::userdata::AnyUserData;
use crate::vm::util::{lua_rust_error, LuaType};
use crate::vm::value::function::Function;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Any<'a> {
    None,
    Nil,
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Buffer(&'a [u8]),
    Function(Function<'a>),
    Table(Table<'a>),
    UserData(AnyUserData<'a>),
    Thread(Thread<'a>),
}

impl Eq for Any<'_> {}

impl Display for Any<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Any::None => f.write_str("<none>"),
            Any::Nil => f.write_str("nil"),
            Any::Number(v) => write!(f, "{}", v),
            Any::Boolean(v) => write!(f, "{}", v),
            Any::String(v) => write!(f, "{}", v),
            Any::Buffer(v) => write!(f, "{:?}", v),
            Any::Function(v) => write!(f, "{}", v),
            Any::Table(v) => write!(f, "{}", v),
            Any::UserData(v) => write!(f, "{}", v),
            Any::Thread(v) => write!(f, "{}", v),
        }
    }
}

impl Any<'_> {
    pub fn ty(&self) -> Type {
        match self {
            Any::None => Type::None,
            Any::Nil => Type::Nil,
            Any::Number(_) => Type::Number,
            Any::Boolean(_) => Type::Boolean,
            Any::String(_) => Type::String,
            Any::Buffer(_) => Type::String,
            Any::Function(_) => Type::Function,
            Any::Table(_) => Type::Table,
            Any::UserData(_) => Type::Userdata,
            Any::Thread(_) => Type::Thread,
        }
    }

    pub fn to_number(&self) -> Result<crate::ffi::lua::RawNumber, Error> {
        match self {
            Any::Number(v) => Ok(*v),
            Any::String(v) => {
                crate::ffi::lua::RawNumber::from_str(v).map_err(|_| Error::ParseFloat)
            }
            _ => Err(Error::Type(TypeError {
                expected: Type::Number,
                actual: self.ty(),
            })),
        }
    }

    pub fn to_integer(&self) -> Result<crate::ffi::lua::RawInteger, Error> {
        match self {
            Any::Number(v) => Ok(*v as _),
            Any::String(v) => {
                crate::ffi::lua::RawInteger::from_str(v).map_err(|_| Error::ParseInt)
            }
            _ => Err(Error::Type(TypeError {
                expected: Type::Number,
                actual: self.ty(),
            })),
        }
    }
}

unsafe impl IntoLua for Any<'_> {
    fn into_lua(self, vm: &Vm) -> u16 {
        match self {
            Any::None => 0,
            Any::Nil => {
                unsafe { lua_pushnil(vm.as_ptr()) };
                1
            }
            Any::Number(v) => v.into_lua(vm),
            Any::Boolean(v) => v.into_lua(vm),
            Any::String(v) => v.into_lua(vm),
            Any::Buffer(v) => v.into_lua(vm),
            Any::Function(v) => v.into_lua(vm),
            Any::Table(v) => v.into_lua(vm),
            Any::UserData(_) => 0,
            Any::Thread(_) => 0,
        }
    }
}

unsafe impl IntoParam for Any<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

impl<'a> FromLua<'a> for Any<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self::from_lua(vm, index).unwrap_unchecked()
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        match ty {
            Type::None => Ok(Any::None),
            Type::Nil => Ok(Any::Nil),
            Type::Boolean => {
                let value = unsafe { lua_toboolean(vm.as_ptr(), index) };
                Ok(Any::Boolean(value == 1))
            }
            Type::LightUserdata => Err(Error::UnsupportedType(ty)),
            Type::Number => {
                let value = unsafe { lua_tonumber(vm.as_ptr(), index) };
                Ok(Any::Number(value))
            }
            Type::String => {
                let buffer: &[u8] = unsafe { FromLua::from_lua_unchecked(vm, index) };
                match std::str::from_utf8(buffer) {
                    Ok(s) => Ok(Any::String(s)),
                    Err(_) => Ok(Any::Buffer(buffer)),
                }
            }
            Type::Table => Ok(unsafe { Any::Table(FromLua::from_lua_unchecked(vm, index)) }),
            Type::Function => {
                Ok(unsafe { Any::Function(FromLua::from_lua_unchecked(vm, index)) })
            }
            Type::Userdata => {
                Ok(unsafe { Any::UserData(FromLua::from_lua_unchecked(vm, index)) })
            }
            Type::Thread => Ok(unsafe { Any::Thread(FromLua::from_lua_unchecked(vm, index)) }),
        }
    }
}

unsafe impl SimpleDrop for Any<'_> {}

impl LuaType for Any<'_> {}

impl<'a> FromParam<'a> for Any<'a> {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        match FromLua::from_lua(vm, index) {
            Ok(v) => v,
            Err(e) => lua_rust_error(vm.as_ptr(), e),
        }
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        FromLua::from_lua(vm, index).ok()
    }
}

/// A marker struct to run lua code which may return any number of values on the stack.
pub struct AnyParam;

impl FromLua<'_> for AnyParam {
    #[inline(always)]
    unsafe fn from_lua_unchecked(_: &Vm, _: i32) -> Self {
        AnyParam
    }

    #[inline(always)]
    fn from_lua(_: &Vm, _: i32) -> crate::vm::Result<Self> {
        Ok(AnyParam)
    }

    #[inline(always)]
    fn num_values() -> i16 {
        -1
    }
}

/// A raw primitive to return arbitrary count of values from a C function.
pub struct UncheckedAnyReturn(i32);

impl UncheckedAnyReturn {
    /// Construct a [UncheckedAnyReturn].
    ///
    /// # Panic
    ///
    /// This function panics when the count of arguments is greater than the lua stack size itself.
    ///
    /// # Safety
    ///
    /// It is UB to run any operation which may alter the lua stack after constructing this
    /// primitive.
    pub unsafe fn new(vm: &Vm, count: i32) -> Self {
        let top = vm.top();
        if count > top as _ {
            panic!()
        }
        UncheckedAnyReturn(count)
    }
}

unsafe impl IntoParam for UncheckedAnyReturn {
    #[inline(always)]
    fn into_param(self, _: &Vm) -> i32 {
        self.0 as _
    }
}

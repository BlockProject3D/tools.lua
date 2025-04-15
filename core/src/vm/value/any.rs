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

use std::fmt::Display;
use crate::ffi::lua::{lua_pushnil, lua_toboolean, lua_tonumber, lua_type, Type};
use crate::util::SimpleDrop;
use crate::vm::error::Error;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::function::LuaFunction;
use crate::vm::table::Table;
use crate::vm::thread::Thread;
use crate::vm::userdata::AnyUserData;
use crate::vm::util::{lua_rust_error, LuaType};
use crate::vm::Vm;

#[derive(Debug, PartialEq, Clone)]
pub enum AnyValue<'a> {
    None,
    Nil,
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Buffer(&'a [u8]),
    Function(LuaFunction<'a>),
    Table(Table<'a>),
    UserData(AnyUserData<'a>),
    Thread(Thread<'a>)
}

impl Eq for AnyValue<'_> { }

impl Display for AnyValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyValue::None => f.write_str("<none>"),
            AnyValue::Nil => f.write_str("nil"),
            AnyValue::Number(v) => write!(f, "{}", v),
            AnyValue::Boolean(v) => write!(f, "{}", v),
            AnyValue::String(v) => write!(f, "{}", v),
            AnyValue::Buffer(v) => write!(f, "{:?}", v),
            AnyValue::Function(v) => write!(f, "{}", v),
            AnyValue::Table(v) => write!(f, "{}", v),
            AnyValue::UserData(v) => write!(f, "{}", v),
            AnyValue::Thread(v) => write!(f, "{}", v)
        }
    }
}

impl AnyValue<'_> {
    pub fn ty(&self) -> Type {
        match self {
            AnyValue::None => Type::None,
            AnyValue::Nil => Type::Nil,
            AnyValue::Number(_) => Type::Number,
            AnyValue::Boolean(_) => Type::Boolean,
            AnyValue::String(_) => Type::String,
            AnyValue::Buffer(_) => Type::String,
            AnyValue::Function(_) => Type::Function,
            AnyValue::Table(_) => Type::Table,
            AnyValue::UserData(_) => Type::Userdata,
            AnyValue::Thread(_) => Type::Thread
        }
    }
}

unsafe impl IntoParam for AnyValue<'_> {
    fn into_param(self, vm: &Vm) -> u16 {
        match self {
            AnyValue::None => 0,
            AnyValue::Nil => {
                unsafe { lua_pushnil(vm.as_ptr()) };
                1
            },
            AnyValue::Number(v) => v.into_lua(vm),
            AnyValue::Boolean(v) => v.into_lua(vm),
            AnyValue::String(v) => v.into_lua(vm),
            AnyValue::Buffer(v) => v.into_lua(vm),
            AnyValue::Function(v) => v.into_lua(vm),
            AnyValue::Table(v) => v.into_lua(vm),
            AnyValue::UserData(_) => 0,
            AnyValue::Thread(_) => 0
        }
    }
}

impl<'a> FromLua<'a> for AnyValue<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self::from_lua(vm, index).unwrap_unchecked()
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        match ty {
            Type::None => Ok(AnyValue::None),
            Type::Nil => Ok(AnyValue::Nil),
            Type::Boolean => {
                let value = unsafe { lua_toboolean(vm.as_ptr(), index) };
                Ok(AnyValue::Boolean(value == 1))
            }
            Type::LightUserdata => Err(Error::UnsupportedType(ty)),
            Type::Number => {
                let value = unsafe { lua_tonumber(vm.as_ptr(), index) };
                Ok(AnyValue::Number(value))
            }
            Type::String => {
                let buffer: &[u8] = unsafe { FromLua::from_lua_unchecked(vm, index) };
                match std::str::from_utf8(buffer) {
                    Ok(s) => Ok(AnyValue::String(s)),
                    Err(_) => Ok(AnyValue::Buffer(buffer))
                }
            }
            Type::Table => Ok(unsafe { AnyValue::Table(FromLua::from_lua_unchecked(vm, index)) }),
            Type::Function => Ok(unsafe { AnyValue::Function(FromLua::from_lua_unchecked(vm, index)) }),
            Type::Userdata => Ok(unsafe { AnyValue::UserData(FromLua::from_lua_unchecked(vm, index)) }),
            Type::Thread => Ok(unsafe { AnyValue::Thread(FromLua::from_lua_unchecked(vm, index)) }),
        }
    }
}

unsafe impl SimpleDrop for AnyValue<'_> {}

impl LuaType for AnyValue<'_> { }

impl<'a> FromParam<'a> for AnyValue<'a> {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        match FromLua::from_lua(vm, index) {
            Ok(v) => v,
            Err(e) => lua_rust_error(vm.as_ptr(), e)
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
pub struct UncheckedAnyReturn(u16);

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
    pub unsafe fn new(vm: &Vm, count: u16) -> Self {
        let top = vm.top();
        if count > top as _ {
            panic!()
        }
        UncheckedAnyReturn(count)
    }
}

unsafe impl IntoParam for UncheckedAnyReturn {
    #[inline(always)]
    fn into_param(self, _: &Vm) -> u16 {
        self.0
    }
}

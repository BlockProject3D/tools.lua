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

use crate::ffi::laux::luaL_testudata;
use crate::ffi::lua::{lua_pushvalue, lua_replace, lua_settop, lua_topointer, lua_touserdata, lua_type, Type};
use crate::vm::error::{Error, TypeError};
use crate::vm::userdata::{UserData, UserDataImmutable};
use crate::vm::value::{FromLua, ImmutableValue, IntoLua};
use crate::vm::Vm;
use std::fmt::{Debug, Display};
use crate::util::core::{AnyStr, SimpleDrop};
use crate::util::LuaFunction;
use crate::vm::table::ImmutableTable;
use crate::vm::util::LuaType;
use crate::vm::value::types::Function;
use crate::vm::value::util::{check_get_metatable, check_push_value};

pub struct AnyUserData<'a> {
    vm: &'a Vm,
    index: i32,
}

impl Clone for AnyUserData<'_> {
    fn clone(&self) -> Self {
        unsafe { lua_pushvalue(self.vm.as_ptr(), self.index) };
        AnyUserData {
            vm: self.vm,
            index: self.vm.top(),
        }
    }
}

impl PartialEq for AnyUserData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl Eq for AnyUserData<'_> {}

//Stupid fmt name in Rust which causes conflicts...
fn internal_display(ud: &AnyUserData, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let res = ud.vm.scope(|_| {
        let res: crate::vm::Result<&str> = ud.call_method("__tostring", ())
            .or_else(|_| ud.call_method("tostring", ()));
        match res {
            Ok(v) => {
                let type_name = ud.get_type_name()?;
                Ok(write!(f, "{}({})", type_name, v))
            },
            Err(e) => Err(e)
        }
    });
    match res {
        Ok(v) => v,
        Err(_) => write!(
            f,
            "userdata@{:X}",
            unsafe { lua_touserdata(ud.vm.as_ptr(), ud.index) } as usize
        )
    }
}

impl Display for AnyUserData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        internal_display(self, f)
    }
}

impl Debug for AnyUserData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserData({:?})", self.index)
    }
}

impl<'a> AnyUserData<'a> {
    /// Creates an AnyUserData from a raw Vm and index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm to link to.
    /// * `index`: the index on the lua stack.
    ///
    /// returns: Table
    ///
    /// # Safety
    ///
    /// Must ensure that index points to a UserData and is absolute. If index is not absolute then
    /// using the produced object is UB. If the index points to any other type then using the produced
    /// object is also UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self { vm, index }
    }

    /// Returns a unique identifier to that table across the Vm it is attached to.
    pub fn uid(&self) -> usize {
        unsafe { lua_topointer(self.vm.as_ptr(), self.index) as _ }
    }

    /// Returns a reference to this UserData value cast to `T`.
    #[inline(always)]
    pub fn get<T: UserData + UserDataImmutable>(&self) -> crate::vm::Result<&T> {
        crate::vm::value::FromLua::from_lua(self.vm, self.index)
    }

    /// Returns a mutable reference to a UserData value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for guaranteeing that only a single reference to this object is
    /// created. That is no other references to this underlying userdata value must exist in Rust
    /// code otherwise using this function is UB.
    pub unsafe fn get_mut<T: UserData>(&mut self) -> crate::vm::Result<&mut T> {
        let this_ptr =
            unsafe { luaL_testudata(self.vm.as_ptr(), self.index, T::CLASS_NAME.as_ptr()) }
                as *mut T;
        if this_ptr.is_null() {
            return Err(Error::Type(TypeError {
                expected: Type::Userdata,
                actual: unsafe { lua_type(self.vm.as_ptr(), self.index) },
            }));
        }
        Ok(unsafe { &mut *this_ptr })
    }

    pub fn get_metatable(&self) -> Option<ImmutableTable> {
        unsafe { check_get_metatable(self.vm, self.index).map(ImmutableTable::from) }
    }

    pub fn get_type_name(&self) -> crate::vm::Result<&str> {
        let tbl = self.get_metatable().ok_or(Error::Type(TypeError {
            expected: Type::Table,
            actual: Type::None,
        }))?;
        let value: &str = tbl.get(c"__metatable")?;
        let value2: &'a str = unsafe { std::mem::transmute(value) };
        unsafe { lua_replace(self.vm.as_ptr(), -2) };
        Ok(value2)
    }

    pub fn call_method<'b, T: FromLua<'b>>(&'b self, name: impl AnyStr, args: impl IntoLua) -> crate::vm::Result<T> {
        let tbl = self.get_metatable().ok_or(Error::Type(TypeError {
            expected: Type::Table,
            actual: Type::None,
        }))?;
        let f: Function = tbl.get(name)?;
        let f = LuaFunction::create(f);
        unsafe { lua_settop(self.vm.as_ptr(), -2) }; // Pop the metatatble from the stack.
        let res: T = f.call(self.vm, (self, args))?;
        f.delete(self.vm);
        Ok(res)
    }
}

impl<'a> FromLua<'a> for AnyUserData<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        AnyUserData::from_raw(vm, vm.get_absolute_index(index))
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        if ty == Type::Userdata {
            Ok(unsafe { AnyUserData::from_raw(vm, vm.get_absolute_index(index)) })
        } else {
            Err(Error::Type(TypeError {
                expected: Type::Userdata,
                actual: ty,
            }))
        }
    }
}

unsafe impl IntoLua for &AnyUserData<'_> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        check_push_value(self.vm, vm, self.index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImmutableAnyUserData<'a>(AnyUserData<'a>);

impl Display for ImmutableAnyUserData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        internal_display(&self.0, f)
    }
}

impl<'a> From<AnyUserData<'a>> for ImmutableAnyUserData<'a> {
    #[inline(always)]
    fn from(value: AnyUserData<'a>) -> Self {
        Self(value)
    }
}

impl<'a> ImmutableAnyUserData<'a> {
    /// Creates an AnyUserData from a raw Vm and index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm to link to.
    /// * `index`: the index on the lua stack.
    ///
    /// returns: Table
    ///
    /// # Safety
    ///
    /// Must ensure that index points to a UserData and is absolute. If index is not absolute then
    /// using the produced object is UB. If the index points to any other type then using the produced
    /// object is also UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self(AnyUserData::from_raw(vm, index))
    }

    /// Returns a unique identifier to that table across the Vm it is attached to.
    #[inline(always)]
    pub fn uid(&self) -> usize {
        self.0.uid()
    }

    /// Returns a reference to this UserData value cast to `T`.
    #[inline(always)]
    pub fn get<T: UserData + UserDataImmutable>(&self) -> crate::vm::Result<&T> {
        self.0.get()
    }

    #[inline(always)]
    pub fn get_metatable(&self) -> Option<ImmutableTable> {
        self.0.get_metatable()
    }

    #[inline(always)]
    pub fn get_type_name(&self) -> crate::vm::Result<&str> {
        self.0.get_type_name()
    }

    #[inline(always)]
    pub fn call_method<'b, T: FromLua<'b> + ImmutableValue>(&'b self, name: impl AnyStr, args: impl IntoLua) -> crate::vm::Result<T> {
        self.0.call_method(name, args)
    }
}

unsafe impl ImmutableValue for ImmutableAnyUserData<'_> {}

impl<'a> FromLua<'a> for ImmutableAnyUserData<'a> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self::from_raw(vm, vm.get_absolute_index(index))
    }

    #[inline(always)]
    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        AnyUserData::from_lua(vm, index).map(Into::into)
    }
}

unsafe impl SimpleDrop for ImmutableAnyUserData<'_> {}
unsafe impl SimpleDrop for AnyUserData<'_> {}
impl LuaType for AnyUserData<'_> {}
impl LuaType for ImmutableAnyUserData<'_> {}

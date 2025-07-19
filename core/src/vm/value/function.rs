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

use crate::ffi::laux::luaL_checktype;
use crate::ffi::lua::{lua_pushvalue, lua_topointer, Type};
use crate::util::core::SimpleDrop;
use crate::vm::core::util::{pcall, push_error_handler};
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::registry::{FromIndex, Set};
use crate::vm::util::LuaType;
use crate::vm::value::util::ensure_type_equals;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use std::fmt::{Debug, Display};

pub struct Function<'a> {
    vm: &'a Vm,
    index: i32,
}

impl Clone for Function<'_> {
    fn clone(&self) -> Self {
        unsafe { lua_pushvalue(self.vm.as_ptr(), self.index) };
        Function {
            vm: self.vm,
            index: self.vm.top(),
        }
    }
}

impl PartialEq for Function<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = unsafe { lua_topointer(self.vm.as_ptr(), self.index) };
        let b = unsafe { lua_topointer(other.vm.as_ptr(), other.index) };
        a == b
    }
}

impl Eq for Function<'_> {}

impl Display for Function<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "function@{:X}",
            unsafe { lua_topointer(self.vm.as_ptr(), self.index) } as usize
        )
    }
}

impl Debug for Function<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LuaFunction({:?})", self.index)
    }
}

unsafe impl SimpleDrop for Function<'_> {}

impl LuaType for Function<'_> {}

impl<'a> FromParam<'a> for Function<'a> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        unsafe { luaL_checktype(vm.as_ptr(), index, Type::Function) };
        Function { vm, index }
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        FromLua::from_lua(vm, index).ok()
    }
}

unsafe impl IntoParam for Function<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

unsafe impl IntoLua for Function<'_> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushvalue(vm.as_ptr(), self.index) };
        1
    }
}

impl Function<'_> {
    pub fn call<'b, R: FromLua<'b>>(&'b self, value: impl IntoLua) -> crate::vm::Result<R> {
        let pos = unsafe { push_error_handler(self.vm.as_ptr()) };
        unsafe {
            lua_pushvalue(self.vm.as_ptr(), self.index);
        }
        let num_values = value.into_lua(self.vm);
        unsafe { pcall(self.vm, num_values as _, R::num_values() as _, pos)? };
        R::from_lua(self.vm, -(R::num_values() as i32))
    }

    /// Returns the absolute index of this function on the Lua stack.
    #[inline(always)]
    pub fn index(&self) -> i32 {
        self.index
    }
}

impl<'a> FromLua<'a> for Function<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Function<'a> {
        Function {
            vm,
            index: vm.get_absolute_index(index),
        }
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        ensure_type_equals(vm, index, Type::Function)?;
        Ok(Function {
            vm,
            index: vm.get_absolute_index(index),
        })
    }
}

impl crate::vm::registry::Value for crate::vm::registry::types::Function {
    type Value<'a> = Function<'a>;

    #[inline(always)]
    unsafe fn from_registry(vm: &Vm, index: i32) -> Self::Value<'_> {
        unsafe { Function::from_lua_unchecked(vm, index) }
    }

    #[inline(always)]
    fn push_registry<R: FromIndex>(value: Self::Value<'_>) -> R {
        unsafe { R::from_index(value.vm, value.index()) }
    }

    #[inline(always)]
    unsafe fn set_registry(key: &impl Set, value: Self::Value<'_>) {
        key.set(value.vm, value.index())
    }
}

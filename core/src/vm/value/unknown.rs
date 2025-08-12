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

use std::fmt::Debug;
use crate::ffi::lua::{lua_replace, lua_type, Type};
use crate::vm::function::IntoParam;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::any::Any;
use crate::vm::value::util::check_value_top;
use crate::vm::Vm;

pub struct Unknown<'a> {
    vm: &'a Vm,
    index: i32
}

impl Debug for Unknown<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Table({:?})", self.index)
    }
}

impl<'a> Unknown<'a> {
    /// Attempts to create an [Unknown] typed value from a specific index on the stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] object this value is attached to.
    /// * `index`: the index of the value on the stack.
    ///
    /// returns: Unknown
    ///
    /// # Safety
    ///
    /// The given stack index must be absolute, if not this is UB. Using this to return the
    /// metatable of an UserData is also UB.
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self {
            vm,
            index
        }
    }

    /// Interprets the underlying reference on the lua stack as the specified Rust type.
    pub fn get<T: FromLua<'a>>(&self) -> crate::vm::Result<T> {
        T::from_lua(self.vm, self.index)
    }

    /// Interprets the underlying reference on the lua stack as the specified Rust type.
    ///
    /// # Safety
    ///
    /// This function assumes the type of the value at index `index` is already of the expected type,
    /// if not, calling this function is UB.
    pub unsafe fn get_unchecked<T: FromLua<'a>>(&self) -> T {
        T::from_lua_unchecked(self.vm, self.index)
    }

    pub fn ty(&self) -> Type {
        unsafe { lua_type(self.vm.as_ptr(), self.index) }
    }

    pub fn to_any(self) -> crate::vm::Result<Any<'a>> {
        Any::from_lua(self.vm, self.index)
    }

    pub fn set(&mut self, value: impl IntoLua) {
        value.into_lua(self.vm);
        unsafe { lua_replace(self.vm.as_ptr(), self.index) };
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

unsafe impl IntoLua for Unknown<'_> {
    fn into_lua(self, vm: &Vm) -> u16 {
        if self.ty() == Type::None { // None is not a value, do not operate the stack or UB.
            return 0; // No value exists on the stack so IntoLua returns 0 values.
        }
        check_value_top(self.vm, vm, self.index)
    }
}

unsafe impl IntoParam for Unknown<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

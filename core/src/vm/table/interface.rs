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
use crate::ffi::lua::{lua_gettop, lua_pushvalue, lua_type, Type};
use crate::util::core::SimpleDrop;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::registry::core::RegistryKey;
use crate::vm::registry::Registry;
use crate::vm::table::Table;
use crate::vm::util::LuaType;
use crate::vm::value::util::{ensure_type_equals, ensure_value_top};
use crate::vm::value::FromLua;
use crate::vm::Vm;

unsafe impl SimpleDrop for Table<'_> {}

impl<'a> FromParam<'a> for Table<'a> {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        luaL_checktype(vm.as_ptr(), index, Type::Table);
        Table::from_raw(vm, index)
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        if unsafe { lua_type(vm.as_ptr(), index) } != Type::Table {
            return None;
        }
        Some(unsafe { Table::from_raw(vm, index) })
    }
}

impl<'a> FromLua<'a> for Table<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Table::from_raw(vm, vm.get_absolute_index(index))
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        ensure_type_equals(vm, index, Type::Table)?;
        Ok(unsafe { Table::from_raw(vm, vm.get_absolute_index(index)) })
    }
}

unsafe impl IntoParam for Table<'_> {
    fn into_param(self, vm: &Vm) -> u16 {
        let top = unsafe { lua_gettop(vm.as_ptr()) };
        if top != self.index() {
            unsafe { lua_pushvalue(vm.as_ptr(), self.index()) };
        }
        1
    }
}

impl LuaType for Table<'_> {}

impl Registry for Table<'_> {
    type RegistryValue = crate::vm::registry::types::Table;

    #[inline(always)]
    fn registry_put(self) -> RegistryKey<Self::RegistryValue> {
        // If the table is not at the top of the stack, move it to the top.
        ensure_value_top(self.vm, self.index());
        unsafe { RegistryKey::from_top(self.vm) }
    }

    #[inline(always)]
    fn registry_swap(
        self,
        old: RegistryKey<Self::RegistryValue>,
    ) -> RegistryKey<Self::RegistryValue> {
        // If the table is not at the top of the stack, move it to the top.
        ensure_value_top(self.vm, self.index());
        unsafe { old.as_raw().replace(self.vm) };
        old
    }
}

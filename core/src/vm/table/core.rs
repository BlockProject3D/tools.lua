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

use crate::ffi::ext::{lua_ext_tab_len, MSize};
use crate::ffi::lua::{lua_createtable, lua_gettable, lua_gettop, lua_objlen, lua_pushvalue, lua_rawseti, lua_setmetatable, lua_settable, lua_topointer};
use crate::vm::table::iter::Iter;
use crate::vm::table::traits::{GetTable, SetTable};
use crate::vm::value::util::{check_get_metatable, check_push_single};
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use std::fmt::{Debug, Display};

pub struct Table<'a> {
    pub(super) vm: &'a Vm,
    index: i32,
}

impl Clone for Table<'_> {
    fn clone(&self) -> Self {
        unsafe { lua_pushvalue(self.vm.as_ptr(), self.index) };
        Table {
            vm: self.vm,
            index: self.vm.top(),
        }
    }
}

impl PartialEq for Table<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl Eq for Table<'_> {}

impl Display for Table<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "table@{:X}",
            self.uid()
        )
    }
}

impl Debug for Table<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Table({:?})", self.index)
    }
}

impl<'a> Table<'a> {
    /// Creates a table from a raw Vm and index.
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
    /// Must ensure that index points to a table and is absolute. If index is not absolute then
    /// using the produced table is UB. If the index points to any other type then using the produced
    /// table is also UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self { vm, index }
    }

    /// Returns a unique identifier to that table across the Vm it is attached to.
    #[inline(always)]
    pub fn uid(&self) -> usize {
        unsafe { lua_topointer(self.vm.as_ptr(), self.index) as _ }
    }

    pub fn new(vm: &'a Vm) -> Self {
        unsafe { lua_createtable(vm.as_ptr(), 0, 0) };
        let index = unsafe { lua_gettop(vm.as_ptr()) };
        Self { vm, index }
    }

    pub fn with_capacity(vm: &'a Vm, array_capacity: usize, non_array_capcity: usize) -> Self {
        unsafe { lua_createtable(vm.as_ptr(), array_capacity as _, non_array_capcity as _) };
        let index = unsafe { lua_gettop(vm.as_ptr()) };
        Self { vm, index }
    }

    pub fn len(&self) -> usize {
        let mut size: MSize = 0;
        let ret = unsafe { lua_ext_tab_len(self.vm.as_ptr(), self.index, &mut size) };
        if ret == 0 {
            return size as _;
        }
        Iter::from_raw(self.vm, self.index).count() as _
    }

    pub fn set_metatable(&mut self, other: Table) {
        other.into_lua(self.vm);
        unsafe { lua_setmetatable(self.vm.as_ptr(), self.index) };
    }

    pub fn get_metatable(&self) -> Option<Table> {
        unsafe { check_get_metatable(self.vm, self.index) }
    }

    /// Returns the absolute index of this table on the Lua stack.
    #[inline(always)]
    pub fn index(&self) -> i32 {
        self.index
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new iterator for this table.
    ///
    /// This function borrows mutably to avoid messing up the Lua stack while iterating.
    pub fn iter(&mut self) -> Iter {
        Iter::from_raw(self.vm, self.index)
    }

    pub fn get<'b, T: FromLua<'b>>(&'b self, key: impl GetTable) -> crate::vm::Result<T> {
        if T::num_values() != 1 {
            return Err(crate::vm::error::Error::MultiValue);
        }
        unsafe {
            key.get_table(self.vm.as_ptr(), self.index)?;
            T::from_lua(self.vm, -1)
        }
    }

    pub fn set(&mut self, key: impl SetTable, value: impl IntoLua) -> crate::vm::Result<()> {
        unsafe {
            check_push_single(self.vm, value)?;
            key.set_table(self.vm.as_ptr(), self.index)?;
        }
        Ok(())
    }

    pub fn get_any<'b, T: FromLua<'b>>(&'b self, key: impl IntoLua) -> crate::vm::Result<T> {
        if T::num_values() != 1 {
            return Err(crate::vm::error::Error::MultiValue);
        }
        unsafe {
            check_push_single(self.vm, key)?;
            lua_gettable(self.vm.as_ptr(), self.index);
            T::from_lua(self.vm, -1)
        }
    }

    pub fn set_any(&mut self, key: impl IntoLua, value: impl IntoLua) -> crate::vm::Result<()> {
        unsafe {
            check_push_single(self.vm, key)?;
            check_push_single(self.vm, value)?;
            lua_settable(self.vm.as_ptr(), self.index);
        }
        Ok(())
    }

    pub fn push(&mut self, value: impl IntoLua) -> crate::vm::Result<()> {
        unsafe {
            let len = lua_objlen(self.vm.as_ptr(), self.index);
            check_push_single(self.vm, value)?;
            lua_rawseti(self.vm.as_ptr(), self.index, len as i32 + 1);
        }
        Ok(())
    }

    pub fn collect<T: FromLua<'a>>(self) -> crate::vm::Result<T> {
        T::from_lua(self.vm, self.index)
    }
}

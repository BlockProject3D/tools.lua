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
use crate::ffi::lua::{lua_createtable, lua_getfield, lua_gettop, lua_pushvalue};
use crate::util::AnyStr;
use crate::vm::core::util::{pcall, push_error_handler};
use crate::vm::table::iter::Iter;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::table::Scope;
use crate::vm::Vm;

pub struct Table<'a> {
    vm: &'a Vm,
    index: i32
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

    #[inline(always)]
    pub fn lock(&mut self) -> Scope {
        Scope::new(self.vm, self.index)
    }

    pub fn len(&self) -> usize {
        let mut size: MSize = 0;
        let ret = unsafe { lua_ext_tab_len(self.vm.as_ptr(), self.index, &mut size) };
        if ret == 0 {
            return size as _;
        }
        Iter::from_raw(self.vm, self.index).count() as _
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

    pub fn call_function<'b, T: IntoLua, R: FromLua<'b>>(&'b self, name: impl AnyStr, value: T) -> crate::vm::Result<R> {
        let pos = unsafe { push_error_handler(self.vm.as_ptr()) };
        unsafe { lua_getfield(self.vm.as_ptr(), self.index, name.to_str()?.as_ptr()) };
        let num_values = value.into_lua(self.vm);
        unsafe { pcall(self.vm, num_values as _, R::num_values() as _, pos)? };
        R::from_lua(self.vm, -(R::num_values() as i32))
    }

    pub fn call_method<'b, T: IntoLua, R: FromLua<'b>>(&'b self, name: impl AnyStr, value: T) -> crate::vm::Result<R> {
        let pos = unsafe { push_error_handler(self.vm.as_ptr()) };
        unsafe { lua_getfield(self.vm.as_ptr(), self.index, name.to_str()?.as_ptr()) };
        unsafe { lua_pushvalue(self.vm.as_ptr(), self.index) };
        let num_values = value.into_lua(self.vm);
        unsafe { pcall(self.vm, (num_values + 1) as _, R::num_values() as _, pos)? };
        R::from_lua(self.vm, -(R::num_values() as i32))
    }

    /// Creates a new iterator for this table.
    ///
    /// This function borrows mutably to avoid messing up the Lua stack while iterating.
    pub fn iter(&mut self) -> Iter {
        Iter::from_raw(self.vm, self.index)
    }
}

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
use crate::vm::table::iter::Iter;
use crate::vm::table::Table;
use crate::vm::table::traits::GetTable;
use crate::vm::value::{FromLua, ImmutableValue, IntoLua};
use crate::vm::Vm;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutableTable<'a>(Table<'a>);

impl Display for ImmutableTable<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Table::fmt(&self.0, f)
    }
}

impl<'a> From<Table<'a>> for ImmutableTable<'a> {
    #[inline(always)]
    fn from(value: Table<'a>) -> Self {
        Self(value)
    }
}

impl<'a> ImmutableTable<'a> {
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
        Self(Table::from_raw(vm, index))
    }

    /// Returns a unique identifier to that table across the Vm it is attached to.
    #[inline(always)]
    pub fn uid(&self) -> usize {
        self.0.uid()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn get_metatable(&self) -> Option<ImmutableTable> {
        self.0.get_metatable().map(ImmutableTable)
    }

    /// Returns the absolute index of this table on the Lua stack.
    #[inline(always)]
    pub fn index(&self) -> i32 {
        self.0.index()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a new iterator for this table.
    ///
    /// This function borrows mutably to avoid messing up the Lua stack while iterating.
    #[inline(always)]
    pub fn iter(&mut self) -> Iter {
        self.0.iter()
    }

    #[inline(always)]
    pub fn get<'b, T: FromLua<'b> + ImmutableValue>(&'b self, key: impl GetTable) -> crate::vm::Result<T> {
        self.0.get(key)
    }

    #[inline(always)]
    pub fn get_any<'b, T: FromLua<'b> + ImmutableValue>(&'b self, key: impl IntoLua) -> crate::vm::Result<T> {
        self.0.get_any(key)
    }

    #[inline(always)]
    pub fn collect<T: FromLua<'a> + ImmutableValue>(self) -> crate::vm::Result<T> {
        self.0.collect()
    }

    /// Returns the underlying Lua Table.
    ///
    /// # Safety
    ///
    /// This function violates the contract of this type and is only intended to be used in order to
    /// send the table to Lua. Note that you must not use this to expose a UserData metatable as
    /// otherwise Lua could override functions like __gc and cause all kinds of UB.
    #[inline(always)]
    pub unsafe fn to_table(self) -> Table<'a> {
        self.0
    }
}

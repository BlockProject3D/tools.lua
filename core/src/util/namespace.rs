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

use bp3d_debug::info;
use crate::ffi::lua::lua_settop;
use crate::util::core::AnyStr;
use crate::vm::registry::core::Key;
use crate::vm::table::Table;
use crate::vm::userdata::{NameConvert, UserData};
use crate::vm::userdata::util::get_static_table;
use crate::vm::value::IntoLua;
use crate::vm::Vm;

pub struct Namespace<'a> {
    vm: &'a Vm,
    table: Table<'a>,
}

impl<'a> Namespace<'a> {
    fn from_table<'b>(
        vm: &'a Vm,
        table: Table<'a>,
        names: impl Iterator<Item = &'b str>,
    ) -> crate::vm::Result<Self> {
        let key = Key::<crate::vm::registry::types::Table>::new(table);
        let key = vm.scope(|vm| {
            for name in names {
                let mut table = key.push(vm);
                let tbl: Option<Table> = table.get(name)?;
                let tab = match tbl {
                    Some(v) => v,
                    None => {
                        table.set(name, Table::new(vm))?;
                        table.get(name)?
                    }
                };
                key.set(tab);
            }
            Ok(key)
        })?;
        let table = key.push(vm);
        key.delete(vm);
        Ok(Self { vm, table })
    }

    pub fn new(vm: &'a Vm, path: &str) -> crate::vm::Result<Self> {
        let mut names = path.split(".");
        let name = names.next().expect("Attempt to build an empty namespace");
        let value: Option<Table<'a>> = vm.get_global(name)?;
        let table = match value {
            Some(table) => table,
            None => {
                vm.set_global(name, Table::new(vm))?;
                vm.get_global(name)?
            }
        };
        Self::from_table(vm, table, names)
    }

    pub fn add<'b, T: IntoLua>(
        &mut self,
        items: impl IntoIterator<Item = (&'b str, T)>,
    ) -> crate::vm::Result<()> {
        for (name, item) in items {
            self.table.set(name, item)?;
        }
        Ok(())
    }

    pub fn add_userdata<T: UserData>(&mut self, name: impl AnyStr, case: impl NameConvert) -> crate::vm::Result<()> {
        info!("Adding userdata type {:?} as {:?}", T::CLASS_NAME, name.to_str()?);
        self.vm.register_userdata::<T>(case)?;
        self.table.set(name, get_static_table::<T>(self.vm)
            .map(|v| unsafe { v.to_table() }))?;
        Ok(())
    }

    pub fn vm(&self) -> &'a Vm {
        self.vm
    }
}

impl Drop for Namespace<'_> {
    fn drop(&mut self) {
        // Clear the table which should be on top of the stack.
        unsafe { lua_settop(self.vm.as_ptr(), -2) };
    }
}

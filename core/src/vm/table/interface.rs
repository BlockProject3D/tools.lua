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

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use crate::ffi::laux::luaL_checktype;
use crate::ffi::lua::{
    lua_getfield, lua_rawgeti, lua_rawseti, lua_setfield, lua_type,
    State, Type,
};
use crate::impl_registry_value;
use crate::util::core::{AnyStr, SimpleDrop};
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::registry::{FromIndex, Set};
use crate::vm::table::traits::{GetTable, SetTable};
use crate::vm::table::{ImmutableTable, Table};
use crate::vm::util::LuaType;
use crate::vm::value::util::{check_type_equals, check_value_top};
use crate::vm::value::{FromLua, ImmutableValue, IntoLua};
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
        check_type_equals(vm, index, Type::Table)?;
        Ok(unsafe { Table::from_raw(vm, vm.get_absolute_index(index)) })
    }
}

impl LuaType for ImmutableTable<'_> {}
unsafe impl SimpleDrop for ImmutableTable<'_> {}
unsafe impl ImmutableValue for ImmutableTable<'_> {}

impl<'a> FromLua<'a> for ImmutableTable<'a> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Self::from_raw(vm, vm.get_absolute_index(index))
    }

    #[inline(always)]
    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        Table::from_lua(vm, index).map(Into::into)
    }
}

impl<'a> FromParam<'a> for ImmutableTable<'a> {
    #[inline(always)]
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        Table::from_param(vm, index).into()
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        Table::try_from_param(vm, index).map(Into::into)
    }
}

unsafe impl IntoParam for Table<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

unsafe impl IntoLua for Table<'_> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        check_value_top(self.vm, vm, self.index())
    }
}

impl LuaType for Table<'_> {}

impl_registry_value!(crate::vm::registry::types::Table => Table);

impl<T: AnyStr> GetTable for T {
    unsafe fn get_table(self, l: State, index: i32) -> crate::vm::Result<()> {
        lua_getfield(l, index, self.to_str()?.as_ptr());
        Ok(())
    }
}

macro_rules! impl_get_set_table {
    ($($t: ty),*) => {
        $(
            impl GetTable for $t {
                unsafe fn get_table(self, l: State, index: i32) -> crate::vm::Result<()> {
                    lua_rawgeti(l, index, self as _);
                    Ok(())
                }
            }

            impl SetTable for $t {
                unsafe fn set_table(self, l: State, index: i32) -> crate::vm::Result<()> {
                    lua_rawseti(l, index, self as _);
                    Ok(())
                }
            }
        )*
    };
}

impl_get_set_table!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);

impl<T: AnyStr> SetTable for T {
    unsafe fn set_table(self, l: State, index: i32) -> crate::vm::Result<()> {
        lua_setfield(l, index, self.to_str()?.as_ptr());
        Ok(())
    }
}

impl<'a, T: 'static> FromLua<'a> for Vec<T> where for<'b> T: FromLua<'b> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut tbl = Table::from_lua_unchecked(vm, index);
        let mut vec = Vec::new();
        for (_, v) in tbl.iter() {
            let vv = v.get_unchecked();
            vec.push(vv);
        }
        vec
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let mut tbl = Table::from_lua(vm, index)?;
        let mut vec = Vec::new();
        for (_, v) in tbl.iter() {
            let vv = v.get()?;
            vec.push(vv);
        }
        Ok(vec)
    }
}

unsafe impl<T: ImmutableValue> ImmutableValue for Vec<T> {}

impl<'a, K: 'static, V: 'static> FromLua<'a> for HashMap<K, V> where for<'b> K: FromLua<'b> + Hash + Eq,
                                                                     for<'b> V: FromLua<'b> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut tbl = Table::from_lua_unchecked(vm, index);
        let mut map = HashMap::new();
        for (k, v) in tbl.iter() {
            let kk = k.get_unchecked();
            let vv = v.get_unchecked();
            map.insert(kk, vv);
        }
        map
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let mut tbl = Table::from_lua(vm, index)?;
        let mut map = HashMap::new();
        for (k, v) in tbl.iter() {
            let kk = k.get()?;
            let vv = v.get()?;
            map.insert(kk, vv);
        }
        Ok(map)
    }
}

unsafe impl<K: ImmutableValue, V: ImmutableValue> ImmutableValue for HashMap<K, V> {}

impl<'a, K: 'static, V: 'static> FromLua<'a> for BTreeMap<K, V> where for<'b> K: FromLua<'b> + Ord,
                                                                     for<'b> V: FromLua<'b> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let mut tbl = Table::from_lua_unchecked(vm, index);
        let mut map = BTreeMap::new();
        for (k, v) in tbl.iter() {
            let kk = k.get_unchecked();
            let vv = v.get_unchecked();
            map.insert(kk, vv);
        }
        map
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let mut tbl = Table::from_lua(vm, index)?;
        let mut map = BTreeMap::new();
        for (k, v) in tbl.iter() {
            let kk = k.get()?;
            let vv = v.get()?;
            map.insert(kk, vv);
        }
        Ok(map)
    }
}

unsafe impl<K: ImmutableValue, V: ImmutableValue> ImmutableValue for BTreeMap<K, V> {}

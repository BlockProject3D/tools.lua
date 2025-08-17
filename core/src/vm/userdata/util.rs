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

use std::ffi::CStr;
use crate::ffi::lua::{lua_getfield, lua_replace, lua_settop, lua_type, Type, REGISTRYINDEX};
use crate::vm::table::ImmutableTable;
use crate::vm::userdata::UserData;
use crate::vm::Vm;

/// Returns the static table attached to the given UserData type.
///
/// The static table contains all static fields and functions added to the type at registration
/// time.
///
/// This function returns None when the UserData identified with type `T` does not have any static
/// members.
///
/// # Arguments
///
/// * `vm`: the [Vm] the UserData type is attached to.
///
/// returns: Option<Table>
pub fn get_static_table<T: UserData>(vm: &Vm) -> Option<ImmutableTable<'_>> {
    get_static_table_by_name(vm, T::CLASS_NAME)
}

/// Returns the static table attached to the given UserData type.
///
/// The static table contains all static fields and functions added to the type at registration
/// time.
///
/// This function returns None when the UserData identified with type `T` does not have any static
/// members.
///
/// # Arguments
///
/// * `vm`: the [Vm] the UserData type is attached to.
/// * `name`: the name of the UserData type.
///
/// returns: Option<ImmutableTable>
pub fn get_static_table_by_name<'a>(vm: &'a Vm, name: &CStr) -> Option<ImmutableTable<'a>> {
    let val = unsafe {
        lua_getfield(vm.as_ptr(), REGISTRYINDEX, name.as_ptr());
        lua_getfield(vm.as_ptr(), -1, c"__static".as_ptr());
        lua_replace(vm.as_ptr(), -2);
        if lua_type(vm.as_ptr(), -1) == Type::Nil {
            // No static table exists on the given userdata object, skip...
            lua_settop(vm.as_ptr(), -2);
            return None;
        }
        ImmutableTable::from_raw(vm, vm.top())
    };
    Some(val)
}

/// Returns the metatable attached to the given UserData type.
///
/// # Arguments
///
/// * `vm`: the [Vm] the UserData type is attached to.
/// * `name`: the name of the UserData type.
///
/// returns: Option<ImmutableTable>
pub fn get_metatable<T: UserData>(vm: &Vm) -> Option<ImmutableTable<'_>> {
    get_metatable_by_name(vm, T::CLASS_NAME)
}

/// Returns the metatable attached to the given UserData type.
///
/// # Arguments
///
/// * `vm`: the [Vm] the UserData type is attached to.
/// * `name`: the name of the UserData type.
///
/// returns: Option<ImmutableTable>
pub fn get_metatable_by_name<'a>(vm: &'a Vm, name: &CStr) -> Option<ImmutableTable<'a>> {
    let val = unsafe {
        lua_getfield(vm.as_ptr(), REGISTRYINDEX, name.as_ptr());
        if lua_type(vm.as_ptr(), -1) == Type::Nil {
            // No metatable exists on the given userdata object, skip...
            lua_settop(vm.as_ptr(), -2);
            return None;
        }
        ImmutableTable::from_raw(vm, vm.top())
    };
    Some(val)
}

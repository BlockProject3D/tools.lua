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
use crate::ffi::lua::Type;
use crate::impl_registry_value;
use crate::util::core::SimpleDrop;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::registry::{FromIndex, Set};
use crate::vm::thread::value::Thread;
use crate::vm::util::LuaType;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::util::{check_type_equals, check_value_top};
use crate::vm::Vm;

impl<'a> FromLua<'a> for Thread<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Thread::from_raw(vm, vm.get_absolute_index(index))
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        check_type_equals(vm, index, Type::Thread)?;
        unsafe { Ok(Thread::from_raw(vm, vm.get_absolute_index(index))) }
    }
}

unsafe impl SimpleDrop for Thread<'_> {}

impl LuaType for Thread<'_> {}

impl<'a> FromParam<'a> for Thread<'a> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        luaL_checktype(vm.as_ptr(), index, Type::Thread);
        Thread::from_raw(vm, vm.get_absolute_index(index))
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        Thread::from_lua(vm, index).ok()
    }
}

unsafe impl IntoParam for Thread<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

unsafe impl IntoLua for Thread<'_> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        check_value_top(self.vm, vm, self.index())
    }
}

impl_registry_value!(crate::vm::registry::types::Thread => Thread);

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

use crate::ffi::lua::Type;
use crate::vm::core::{pcall, push_error_handler};
use crate::vm::registry::core::RegistryKey;
use crate::vm::registry::Register;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::util::{ensure_type_equals, ensure_value_top};
use crate::vm::Vm;

pub struct LuaFunction<'a> {
    vm: &'a Vm,
    index: i32
}

impl<'a> LuaFunction<'a> {
    pub fn call<'b, T: IntoLua, R: FromLua<'b>>(&'b self, value: T) -> crate::vm::Result<R> {
        let pos = push_error_handler(self.vm.as_ptr());
        let num_values = value.into_lua(self.vm)?;
        pcall(self.vm, num_values as _, R::num_values() as _, pos)?;
        R::from_lua(self.vm, -(R::num_values() as i32))
    }
}

impl<'a> FromLua<'a> for LuaFunction<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> LuaFunction<'a> {
        LuaFunction {
            vm,
            index: vm.get_absolute_index(index)
        }
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        ensure_type_equals(vm, index, Type::Function)?;
        Ok(LuaFunction { vm, index: vm.get_absolute_index(index) })
    }
}

impl Register for LuaFunction<'_> {
    type RegistryValue = crate::vm::registry::types::LuaFunction;

    fn register(self, vm: &Vm) -> RegistryKey<Self::RegistryValue> {
        // If the function is not at the top of the stack, move it to the top.
        ensure_value_top(vm, self.index);
        unsafe { RegistryKey::from_top(vm) }
    }
}

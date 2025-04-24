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

use crate::util::core::AnyStr;
use crate::vm::core::util::{pcall, push_error_handler};
use crate::vm::registry::core::Key;
use crate::vm::registry::types::{Function, Table};
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;

pub struct LuaMethod {
    obj: Key<Table>,
    method: Key<Function>
}

impl LuaMethod {
    pub fn create(obj: crate::vm::table::Table, method_name: impl AnyStr) -> crate::vm::Result<Self> {
        let method: crate::vm::value::Function = obj.get_field(method_name)?;
        Ok(Self { method: Key::new(method), obj: Key::new(obj) })
    }

    pub fn call<'a, R: FromLua<'a>>(&self, vm: &'a Vm, value: impl IntoLua) -> crate::vm::Result<R> {
        let pos = unsafe { push_error_handler(vm.as_ptr()) };
        unsafe { self.method.as_raw().push(vm) };
        unsafe { self.obj.as_raw().push(vm) };
        let num_values = value.into_lua(vm);
        unsafe { pcall(vm, (num_values + 1) as _, R::num_values() as _, pos)? };
        R::from_lua(vm, -(R::num_values() as i32))
    }

    pub fn delete(self, vm: &Vm) {
        self.method.delete(vm);
        self.obj.delete(vm);
    }
}

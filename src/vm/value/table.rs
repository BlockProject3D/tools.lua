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
use crate::ffi::lua::{lua_getfield, lua_gettop, lua_setfield, lua_settop, lua_type, Type};
use crate::vm::function::FromParam;
use crate::vm::Vm;
use crate::vm::error::TypeError;
use crate::vm::util::AnyStr;
use crate::vm::value::{FromLua, IntoLua};

#[derive(Copy, Clone)]
pub struct Table<'a> {
    vm: &'a Vm,
    index: i32
}

pub struct Scope<'a> {
    vm: &'a Vm,
    index: i32,
    initial_top: i32
}

impl<'a> Scope<'a> {
    fn new(vm: &'a Vm, index: i32) -> Self {
        let initial_top = unsafe { lua_gettop(vm.as_ptr()) };
        Self { vm, index, initial_top }
    }

    pub fn set_field(&mut self, name: impl AnyStr, value: impl IntoLua) -> crate::vm::Result<()> {
        unsafe {
            let nums = value.into_lua(self.vm)?;
            if nums > 1 {
                // Clear the stack.
                lua_settop(self.vm.as_ptr(), -(nums as i32)-1);
                //FIXME: Better error type
                return Err(crate::vm::error::Error::Unknown)
            }
            lua_setfield(self.vm.as_ptr(), self.index, name.to_str()?.as_ptr());
        }
        Ok(())
    }

    pub fn get_field<'b, T: FromLua<'b>>(&'b self, name: impl AnyStr) -> crate::vm::Result<T> {
        if T::num_values() > 1 {
            //FIXME: Better error type
            return Err(crate::vm::error::Error::Unknown)
        }
        unsafe {
            lua_getfield(self.vm.as_ptr(), self.index, name.to_str()?.as_ptr());
            T::from_lua(self.vm, -1)
        }
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        let top = unsafe { lua_gettop(self.vm.as_ptr()) };
        let count = top - self.initial_top;
        // Pop count values off the stack to ensure the stack is cleared after all table
        // manipulations are finished.
        unsafe { lua_settop(self.vm.as_ptr(), -count-1) };
    }
}

impl<'a> Table<'a> {
    pub fn lock(&mut self) -> Scope {
        Scope::new(self.vm, self.index)
    }
}

impl<'a> FromParam<'a> for Table<'a> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        luaL_checktype(vm.as_ptr(), index, Type::Table);
        Table {
            vm,
            index
        }
    }
}

impl<'a> FromLua<'a> for Table<'a> {
    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let ty = unsafe { lua_type(vm.as_ptr(), index) };
        if ty == Type::Table {
            Ok(Table { vm, index })
        } else {
            Err(crate::vm::error::Error::Type(TypeError {
                expected: Type::Table,
                actual: ty
            }))
        }
    }
}

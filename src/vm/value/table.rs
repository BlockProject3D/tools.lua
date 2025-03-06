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
use crate::ffi::lua::{lua_getfield, lua_gettop, lua_setfield, lua_settop, Type};
use crate::vm::function::FromParam;
use crate::vm::{LuaState, Stack};
use crate::vm::util::AnyStr;
use crate::vm::value::{FromLua, IntoLua};

#[derive(Copy, Clone)]
pub struct Table<'a> {
    vm: &'a LuaState,
    index: i32
}

pub struct Scope<'a> {
    vm: &'a LuaState,
    index: i32,
    initial_top: i32
}

impl<'a> Scope<'a> {
    fn new(vm: &'a LuaState, index: i32) -> Self {
        let initial_top = unsafe { lua_gettop(**vm) };
        Self { vm, index, initial_top }
    }

    pub fn set_field(&mut self, name: impl AnyStr, value: impl IntoLua) -> crate::vm::Result<()> {
        unsafe {
            let nums = value.into_lua(self.vm)?;
            if nums > 1 {
                // Clear the stack.
                lua_settop(**self.vm, -(nums as i32)-1);
                //FIXME: Better error type
                return Err(crate::vm::error::Error::Unknown)
            }
            lua_setfield(**self.vm, self.index, name.to_str()?.as_ptr());
        }
        Ok(())
    }

    pub fn get_field<'b, T: FromLua<'b>>(&'b self, name: impl AnyStr) -> crate::vm::Result<T> {
        if T::num_values() > 1 {
            //FIXME: Better error type
            return Err(crate::vm::error::Error::Unknown)
        }
        unsafe {
            lua_getfield(**self.vm, self.index, name.to_str()?.as_ptr());
            T::from_lua(self.vm, -1)
        }
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        let top = unsafe { lua_gettop(**self.vm) };
        let count = top - self.initial_top;
        // Pop count values off the stack to ensure the stack is cleared after all table
        // manipulations are finished.
        unsafe { lua_settop(**self.vm, -count-1) };
    }
}

impl<'a> Table<'a> {
    pub fn lock(&mut self) -> Scope {
        Scope::new(self.vm, self.index)
    }
}

impl<'a> FromParam<'a> for Table<'a> {
    unsafe fn from_param(stack: &'a Stack) -> Self {
        let index = stack.pop();
        luaL_checktype(stack.as_ptr(), index, Type::Table);
        Table {
            vm: stack.as_state(),
            index
        }
    }
}

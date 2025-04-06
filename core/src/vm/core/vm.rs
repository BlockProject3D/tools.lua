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

use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use bp3d_debug::debug;
use crate::ffi::laux::{luaL_newstate, luaL_openlibs};
use crate::ffi::lua::{lua_close, lua_getfield, lua_gettop, lua_pushnil, lua_remove, lua_setfield, lua_settop, State, ThreadStatus, GLOBALSINDEX, REGISTRYINDEX};
use crate::util::AnyStr;
use crate::vm::core::{Load, LoadString};
use crate::vm::core::util::{handle_syntax_error, pcall, push_error_handler};
use crate::vm::error::Error;
use crate::vm::userdata::core::Registry;
use crate::vm::userdata::UserData;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::function::LuaFunction;

pub struct Vm {
    l: State
}

impl Vm {
    #[inline(always)]
    pub unsafe fn from_raw(l: State) -> Self {
        Self {
            l
        }
    }

    pub fn scope<R: 'static, F: FnOnce(&Vm) -> crate::vm::Result<R>>(&self, f: F) -> crate::vm::Result<R> {
        let top = self.top();
        let r = f(self)?;
        unsafe { lua_settop(self.l, top) };
        Ok(r)
    }

    pub fn register_userdata<T: UserData>(&self) -> crate::vm::Result<()> {
        let reg = unsafe { Registry::<T>::new(self) }.map_err(Error::UserData)?;
        let res = T::register(&reg).map_err(Error::UserData);
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                unsafe {
                    lua_pushnil(self.l);
                    lua_setfield(self.l, REGISTRYINDEX, T::CLASS_NAME.as_ptr());
                }
                Err(e)
            }
        }
    }

    /// Returns the absolute stack index for the given index.
    #[inline(always)]
    pub fn get_absolute_index(&self, index: i32) -> i32 {
        if index < 0 {
            unsafe { lua_gettop(self.l) + index + 1 }
        } else {
            index
        }
    }

    /// Returns the top of the lua stack.
    #[inline(always)]
    pub fn top(&self) -> i32 {
        unsafe { lua_gettop(self.l) }
    }

    /// Clears the lua stack.
    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe { lua_settop(self.l, 0); }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> State {
        self.l
    }

    pub fn set_global(&self, name: impl AnyStr, value: impl IntoLua) -> crate::vm::Result<()> {
        value.into_lua(self);
        unsafe { lua_setfield(self.as_ptr(), GLOBALSINDEX, name.to_str()?.as_ptr()); }
        Ok(())
    }

    pub fn get_global<'a, R: FromLua<'a>>(&'a self, name: impl AnyStr) -> crate::vm::Result<R> {
        unsafe { lua_getfield(self.as_ptr(), GLOBALSINDEX, name.to_str()?.as_ptr()); }
        R::from_lua(self, -1)
    }

    pub fn run_code<'a, R: FromLua<'a>>(&'a self, code: impl LoadString) -> crate::vm::Result<R> {
        let l = self.as_ptr();
        unsafe {
            // Push error handler and the get the stack position of it.
            let handler_pos = push_error_handler(l);
            // Push the lua code.
            let res = code.load_string(l);
            if res != ThreadStatus::Ok {
                lua_remove(l, handler_pos);
            }
            handle_syntax_error(self, res)?;
            pcall(self, 0, R::num_values() as _, handler_pos)?;
        }
        // Read and return the result of the function from the stack.
        FromLua::from_lua(self, -(R::num_values() as i32))
    }

    pub fn load_code(&self, code: impl LoadString) -> crate::vm::Result<LuaFunction> {
        let l = self.as_ptr();
        unsafe {
            // Push the lua code.
            let res = code.load_string(l);
            handle_syntax_error(self, res)?;
            Ok(FromLua::from_lua_unchecked(self, -1))
        }
    }

    pub fn run<'a, R: FromLua<'a>>(&'a self, obj: impl Load) -> crate::vm::Result<R> {
        let l = self.as_ptr();
        let handler_pos = unsafe { push_error_handler(l) };
        let res = obj.load(l);
        unsafe {
            if res != ThreadStatus::Ok {
                lua_remove(l, handler_pos);
            }
            handle_syntax_error(self, res)?;
            pcall(self, 0, R::num_values() as _, handler_pos)?;
        }
        // Read and return the result of the function from the stack.
        FromLua::from_lua(self, -(R::num_values() as i32))
    }

    pub fn load(&self, obj: impl Load) -> crate::vm::Result<LuaFunction> {
        let l = self.as_ptr();
        let res = obj.load(l);
        unsafe {
            handle_syntax_error(self, res)?;
            Ok(FromLua::from_lua_unchecked(self, -1))
        }
    }
}

thread_local! {
    static HAS_VM: Cell<bool> = Cell::new(false);
}

pub struct RootVm {
    vm: Vm,
    leaked: Vec<Box<dyn FnOnce()>>
}

impl RootVm {
    pub fn new() -> RootVm {
        if HAS_VM.with(|v| v.get()) {
            panic!("A VM already exists for this thread.")
        }
        let l = unsafe { luaL_newstate() };
        unsafe { luaL_openlibs(l) };
        HAS_VM.set(true);
        RootVm {
            vm: unsafe { Vm::from_raw(l) },
            leaked: Vec::new()
        }
    }

    pub fn attach_box<T: 'static>(&mut self, bx: Box<T>) -> *mut T {
        let ptr = Box::into_raw(bx);
        self.leaked.push(Box::new(move || {
            unsafe { drop(Box::from_raw(ptr)) };
        }));
        ptr
    }
}

impl Deref for RootVm {
    type Target = Vm;

    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}

impl DerefMut for RootVm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}

impl Drop for RootVm {
    fn drop(&mut self) {
        debug!("Deleting leaked pointers...");
        let v = std::mem::replace(&mut self.leaked, Vec::new());
        for f in v {
            f()
        }
        unsafe {
            debug!("Closing Lua VM...");
            lua_close(self.vm.as_ptr());
        }
        HAS_VM.set(false);
    }
}

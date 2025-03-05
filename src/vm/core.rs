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
use std::ffi::c_int;
use crate::ffi::laux::{luaL_callmeta, luaL_newstate, luaL_openlibs, luaL_traceback};
use crate::ffi::lua::{lua_close, lua_gettop, lua_isstring, lua_pcall, lua_pushcclosure, lua_pushlstring, lua_remove, lua_tolstring, lua_type, State, ThreadStatus, Type};
use crate::vm::error::{Error, RuntimeError};
use crate::vm::util::LoadCode;
use crate::vm::value::FromLua;

pub struct Stack {
    l: State,
    index: Cell<i32>
}

impl Stack {
    /// Creates a new [Stack] by wrapping an existing lua VM.
    ///
    /// # Arguments
    ///
    /// * `l`: the raw lua state to wrap.
    /// * `start`: the index at which to start reading values from the lua stack.
    ///
    /// returns: Stack
    ///
    /// # Safety
    ///
    /// This struct SHALL only exist in a [CFunction](crate::ffi::lua::CFunction). Usage in any other
    /// context is UB.
    pub unsafe fn wrap(l: State, start: i32) -> Stack {
        Stack {
            l,
            index: Cell::new(start)
        }
    }

    pub fn as_ptr(&self) -> State {
        self.l
    }

    pub fn pop(&self) -> i32 {
        let i = self.index.get();
        self.index.set(i + 1);
        i
    }
}

const TRACEBACK_NONE: &[u8] = b"<unknown error>\n<no traceback>";
extern "C-unwind" fn error_handler(l: State) -> c_int {
    unsafe {
        let ty = lua_type(l, 1);
        if ty != Type::String {
            // Non-string error object? Try metamethod.
            if (ty == Type::Nil || ty == Type::None) ||
                luaL_callmeta(l, 1, c"__tostring".as_ptr()) != 1 ||
                lua_isstring(l, -1) != 1 {
                // Object does not turn into a string remove it alongside the return value of
                // __tostring.
                lua_remove(l, 1);
                lua_remove(l, 1);
                // Push a place-holder string to avoid the rust code from crashing because the stack
                // would be empty otherwise.
                lua_pushlstring(l, TRACEBACK_NONE.as_ptr() as _, TRACEBACK_NONE.len());
                return 1;
            }
            // Remove the object from the stack so that error message becomes now index 1.
            lua_remove(l, 1);
        }
        // Call traceback with the actual error message as a string which should push onto the stack
        // the stacktrace as a string.
        luaL_traceback(l, l, lua_tolstring(l, 1, std::ptr::null_mut()), 1);
        // Remove the original error message string from the stack.
        lua_remove(l, 1);
        1
    }
}

pub struct Vm {
    l: State
}

impl Vm {
    pub fn new() -> Vm {
        let l = unsafe { luaL_newstate() };
        unsafe { luaL_openlibs(l) };
        Vm {
            l
        }
    }

    pub fn as_ptr(&self) -> State {
        self.l
    }

    pub fn run_code<'a, R: FromLua<'a>>(&'a mut self, code: impl LoadCode) -> crate::vm::Result<R> {
        let l = self.as_ptr();
        // Push error handler and the get the stack position of it.
        let handler_pos = unsafe {
            lua_pushcclosure(l, error_handler, 0);
            lua_gettop(l)
        };
        // Push the lua code.
        let res = code.load_code(l);
        if res != ThreadStatus::Ok {
            unsafe { lua_remove(l, handler_pos) };
        }
        match res {
            ThreadStatus::Ok => (),
            ThreadStatus::ErrSyntax => {
                // If we've got an error, read it and clear the stack.
                let str: &str = FromLua::from_lua(self, -1)?;
                unsafe { lua_remove(l, -1) };
                return Err(Error::Syntax(str.into()))
            }
            ThreadStatus::ErrMem => return Err(Error::Memory),
            _ => return Err(Error::Unknown)
        };
        unsafe {
            // Call the function created by load_code.
            let res = lua_pcall(l, 0, R::num_values() as _, handler_pos);
            // At this point the stack should no longer have the function but still has the error
            // handler and R::num_values results.
            // First remove error handler as we no longer need it.
            lua_remove(l, handler_pos);
            match res {
                ThreadStatus::Ok => (),
                ThreadStatus::ErrRun => {
                    // We've got a runtime error when executing the function so read the full stack
                    // trace produced by luaL_traceback and remove it from the stack.
                    let full_traceback: &str = FromLua::from_lua(self, -1)?;
                    lua_remove(l, -1);
                    return Err(Error::Runtime(RuntimeError::new(full_traceback.into())));
                }
                ThreadStatus::ErrMem => return Err(Error::Memory),
                ThreadStatus::ErrErr => return Err(Error::Error),
                _ => return Err(Error::Unknown)
            };
        }
        // Read and return the result of the function from the stack.
        FromLua::from_lua(self, -1)
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        unsafe {
            println!("Closing Lua VM...");
            lua_close(self.l);
        }
    }
}

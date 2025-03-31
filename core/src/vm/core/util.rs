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

use std::ffi::{c_int, CStr};
use bp3d_util::format::FixedBufStr;
use crate::ffi::laux::{luaL_callmeta, luaL_traceback};
use crate::ffi::lua::{lua_gettop, lua_isstring, lua_pcall, lua_pushcclosure, lua_pushlstring, lua_remove, lua_tolstring, lua_type, State, ThreadStatus, Type};
use crate::vm::error::{Error, RuntimeError};
use crate::vm::value::FromLua;
use crate::vm::Vm;
use crate::ffi::lua::IDSIZE;

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

/// Pushes the error handler on the Lua stack and return its absolute stack index.
///
/// The error handler replaces the error message by a full traceback using [luaL_traceback].
///
/// # Arguments
///
/// * `l`: the lua State on which to push the error handler function.
///
/// returns: i32
///
/// # Safety
///
/// You must ensure that the error handler function is NEVER called outside the context of an error.
#[inline(always)]
pub unsafe fn push_error_handler(l: State) -> c_int {
    unsafe {
        lua_pushcclosure(l, error_handler, 0);
        lua_gettop(l)
    }
}

/// Calls the lua function at the top of the stack in a protected environment.
///
/// # Arguments
///
/// * `vm`: the [Vm] instance on which to call the function.
/// * `nargs`: the number of arguments push on top of the stack.
/// * `nreturns`: the number of returns expected from the function call.
/// * `handler_pos`: the absolute position of the handler on the stack.
///
/// returns: Result<(), Error>
///
/// # Safety
///
/// This function shall not be used without [push_error_handler]. This is also UB if `nargs` does
/// not match the count of arguments push on top of the stack. If the error handler is not the first
/// item on the stack, before function and function arguments, this is UB.
pub unsafe fn pcall(vm: &Vm, nargs: c_int, nreturns: c_int, handler_pos: c_int) -> crate::vm::Result<()> {
    let l = vm.as_ptr();
    unsafe {
        // Call the function created by load_code.
        let res = lua_pcall(l, nargs, nreturns, handler_pos);
        // At this point the stack should no longer have the function but still has the error
        // handler and R::num_values results.
        // First remove error handler as we no longer need it.
        lua_remove(l, handler_pos);
        match res {
            ThreadStatus::Ok => Ok(()),
            ThreadStatus::ErrRun => {
                // We've got a runtime error when executing the function so read the full stack
                // trace produced by luaL_traceback and remove it from the stack.
                let full_traceback: &str = FromLua::from_lua(vm, -1)?;
                lua_remove(l, -1);
                Err(Error::Runtime(RuntimeError::new(full_traceback.into())))
            }
            ThreadStatus::ErrMem => Err(Error::Memory),
            ThreadStatus::ErrErr => Err(Error::Error),
            _ => Err(Error::Unknown)
        }
    }
}

/// Handles a syntax error. A syntax error is an error which may occur as part of a lua_load family
/// of functions.
///
/// # Arguments
///
/// * `vm`: the [Vm] instance which has produced the syntax error.
/// * `res`: the result of the load family of function.
/// * `handler_pos`: the absolute position of the error handler on the stack.
///
/// returns: Result<(), Error>
///
/// # Safety
///
/// Calling this function with a `handler_pos` which does not correspond to the actual error handler
/// C function is UB. This is also UB if the res is not the result of a load function.
pub unsafe fn handle_syntax_error(vm: &Vm, res: ThreadStatus, handler_pos: c_int) -> crate::vm::Result<()> {
    if res != ThreadStatus::Ok {
        unsafe { lua_remove(vm.as_ptr(), handler_pos) };
    }
    match res {
        ThreadStatus::Ok => Ok(()),
        ThreadStatus::ErrSyntax => {
            // If we've got an error, read it and clear the stack.
            let str: &str = FromLua::from_lua(vm, -1)?;
            unsafe { lua_remove(vm.as_ptr(), -1) };
            Err(Error::Syntax(str.into()))
        }
        ThreadStatus::ErrRun => {
            // If we've got an error, read it and clear the stack.
            let str: &str = FromLua::from_lua(vm, -1)?;
            unsafe { lua_remove(vm.as_ptr(), -1) };
            Err(Error::Loader(str.into()))
        }
        ThreadStatus::ErrMem => Err(Error::Memory),
        _ => Err(Error::Unknown)
    }
}

pub struct ChunkNameBuilder {
    inner: FixedBufStr<59>
}

impl ChunkNameBuilder {
    pub fn new() -> Self {
        Self { inner: FixedBufStr::new() }
    }

    pub fn build(self) -> ChunkName {
        let mut buf = FixedBufStr::<IDSIZE>::new();
        unsafe {
            buf.write(self.inner.str().as_bytes());
            buf.write(b"\0");
        }
        ChunkName { inner: buf }
    }
}

impl std::fmt::Write for ChunkNameBuilder {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        self.inner.write_str(s)
    }
}

pub struct ChunkName {
    inner: FixedBufStr<60>
}

impl ChunkName {
    pub fn cstr(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.inner.str().as_bytes()) }
    }
}

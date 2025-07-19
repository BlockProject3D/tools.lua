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

use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use crate::ffi::laux::luaL_error;
use crate::ffi::lua::{lua_isyieldable, lua_remove, lua_resume, lua_status, lua_yield, ThreadStatus};
use crate::vm::error::{Error, RuntimeError};
use crate::vm::function::IntoParam;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Yielded,
    Finished,
}

pub struct Thread<'a> {
    vm: Vm,
    useless: PhantomData<&'a ()>
}

impl PartialEq for Thread<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl Eq for Thread<'_> {}

impl Display for Thread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thread@{:X}", self.uid())
    }
}

impl Debug for Thread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Thread")
    }
}

impl<'a> Thread<'a> {
    /// Creates a thread object from an existing lua thread stack.
    ///
    /// # Arguments
    ///
    /// * `l`: the existing raw lua [State](crate::ffi::lua::State).
    ///
    /// returns: Thread
    ///
    /// # Safety
    ///
    /// Must ensure that l is a valid lua thread stack. If not, the resulting object is UB.
    #[inline(always)]
    pub unsafe fn from_raw(l: crate::ffi::lua::State) -> Self {
        Self {
            vm: Vm::from_raw(l),
            useless: PhantomData
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> crate::ffi::lua::State {
        self.vm.as_ptr()
    }

    /// Returns a unique identifier to that table across the Vm it is attached to.
    #[allow(clippy::missing_transmute_annotations)]
    #[inline(always)]
    pub fn uid(&self) -> usize {
        unsafe { std::mem::transmute(self.vm.as_ptr()) }
    }

    #[inline(always)]
    pub fn status(&self) -> ThreadStatus {
        unsafe { lua_status(self.vm.as_ptr()) }
    }

    pub fn resume(&self, args: impl IntoLua) -> crate::vm::Result<State> {
        let num = args.into_lua(&self.vm);
        let res = unsafe { lua_resume(self.vm.as_ptr(), num as _) };
        match res {
            ThreadStatus::Ok => Ok(State::Finished),
            ThreadStatus::Yield => Ok(State::Yielded),
            ThreadStatus::ErrRun => {
                // We've got a runtime error when executing the function.
                // TODO: In the future, might be great to traceback the thread as well.
                let error_message: &str = FromLua::from_lua(&self.vm, -1)?;
                unsafe { lua_remove(self.vm.as_ptr(), -1) };
                Err(Error::Runtime(RuntimeError::new(
                    String::from(error_message) + "\n<traceback not available>",
                )))
            }
            ThreadStatus::ErrMem => Err(Error::Memory),
            ThreadStatus::ErrErr => Err(Error::Error),
            _ => std::unreachable!(),
        }
    }
}

pub struct Yield;

unsafe impl IntoParam for Yield {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> u16 {
        unsafe {
            if lua_isyieldable(vm.as_ptr()) != 1 {
                luaL_error(vm.as_ptr(), c"attempt to yield a non-thread stack object".as_ptr());
            }
            lua_yield(vm.as_ptr(), 0);
            0
        }
    }
}

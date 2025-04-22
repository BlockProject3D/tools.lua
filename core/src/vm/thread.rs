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
use crate::ffi::lua::{lua_remove, lua_resume, lua_status, lua_tothread, ThreadStatus, Type};
use crate::util::SimpleDrop;
use crate::vm::core::LoadString;
use crate::vm::error::{Error, RuntimeError};
use crate::vm::function::FromParam;
use crate::vm::util::LuaType;
use crate::vm::value::util::ensure_type_equals;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

pub enum State {
    Yielded,
    Finished,
}

pub struct Thread<'a> {
    vm: Vm,
    useless: PhantomData<&'a ()>,
}

impl Clone for Thread<'_> {
    fn clone(&self) -> Self {
        Self {
            vm: unsafe { Vm::from_raw(self.vm.as_ptr()) },
            useless: PhantomData,
        }
    }
}

impl PartialEq for Thread<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.vm.as_ptr() == other.vm.as_ptr()
    }
}

impl Eq for Thread<'_> {}

impl Display for Thread<'_> {
    #[allow(clippy::missing_transmute_annotations)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thread@{:X}", unsafe {
            std::mem::transmute::<_, usize>(self.vm.as_ptr())
        })
    }
}

impl Debug for Thread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Thread")
    }
}

impl Thread<'_> {
    #[inline(always)]
    pub fn run_code<'b, R: FromLua<'b>>(&'b self, code: impl LoadString) -> crate::vm::Result<R> {
        self.vm.run_code(code)
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
                // We've got a runtime error when executing the function so read the full stack
                // trace produced by luaL_traceback and remove it from the stack.
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

impl<'a> FromLua<'a> for Thread<'a> {
    #[inline(always)]
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        Thread {
            vm: Vm::from_raw(lua_tothread(vm.as_ptr(), index)),
            useless: PhantomData,
        }
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        ensure_type_equals(vm, index, Type::Thread)?;
        Ok(Thread {
            vm: unsafe { Vm::from_raw(lua_tothread(vm.as_ptr(), index)) },
            useless: PhantomData,
        })
    }
}

unsafe impl SimpleDrop for Thread<'_> {}

impl LuaType for Thread<'_> {}

impl<'a> FromParam<'a> for Thread<'a> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        luaL_checktype(vm.as_ptr(), index, Type::Thread);
        Thread {
            vm: unsafe { Vm::from_raw(lua_tothread(vm.as_ptr(), index)) },
            useless: PhantomData,
        }
    }

    #[inline(always)]
    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        FromLua::from_lua(vm, index).ok()
    }
}

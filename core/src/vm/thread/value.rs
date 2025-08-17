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
use crate::ffi::lua::{lua_newthread, lua_pushvalue, lua_tothread, lua_type, lua_xmove, ThreadStatus, Type};
use crate::vm::thread::core;
use crate::vm::value::types::Function;
use crate::vm::value::util::move_value_top;
use crate::vm::Vm;

/// Represents a thread object value on a lua stack.
pub struct Thread<'a> {
    pub(super) vm: &'a Vm,
    index: i32,
    thread: core::Thread<'static>
}

impl Clone for Thread<'_> {
    fn clone(&self) -> Self {
        unsafe { lua_pushvalue(self.vm.as_ptr(), self.index) };
        Thread {
            vm: self.vm,
            index: self.vm.top(),
            thread: unsafe { core::Thread::from_raw(self.thread.as_ptr()) }
        }
    }
}

impl PartialEq for Thread<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.thread.eq(&other.thread)
    }
}

impl Eq for Thread<'_> {}

impl Display for Thread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thread@{:X}", self.thread.uid())
    }
}

impl Debug for Thread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread({:?})", self.index)
    }
}

impl<'a> Thread<'a> {
    /// Creates a thread value from a raw Vm and index on `vm` stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm to link to.
    /// * `index`: the index on the lua stack.
    ///
    /// returns: Table
    ///
    /// # Safety
    ///
    /// Must ensure that index points to a thread value and is absolute. If index is not absolute
    /// then using the produced thread value is UB. If the index points to any other type then
    /// using the produced thread value is also UB.
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self {
            vm,
            index,
            thread: core::Thread::from_raw(lua_tothread(vm.as_ptr(), index))
        }
    }

    pub fn new(vm: &'a Vm) -> Self {
        let thread = unsafe { core::Thread::from_raw(lua_newthread(vm.as_ptr())) };
        Self {
            vm,
            index: vm.top(),
            thread
        }
    }

    pub fn set_function(&self, function: Function<'a>) -> crate::vm::Result<()> {
        if self.thread.status() != ThreadStatus::Ok {
            return Err(crate::vm::error::Error::BadThreadState);
        }
        move_value_top(self.vm, function.index());
        unsafe { lua_xmove(self.vm.as_ptr(), self.thread.as_ptr(), 1); }
        unsafe {
            assert_eq!(lua_type(self.thread.as_ptr(), -1), Type::Function);
        };
        Ok(())
    }

    /// Returns the absolute index of this table on the Lua stack.
    #[inline(always)]
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Returns the thread stack object attached to this thread value.
    #[inline(always)]
    pub fn as_thread(&self) -> &core::Thread<'a> {
        //TODO: Check if this is safe as thread lifetime duration should be as long as thread value
        // on the Lua stack; the same goes for ImmutableThread.
        &self.thread
    }
}

/// Represents a thread object value on a lua stack.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ImmutableThread<'a>(Thread<'a>);

impl Display for ImmutableThread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "thread@{:X}", self.0.thread.uid())
    }
}

impl<'a> From<Thread<'a>> for ImmutableThread<'a> {
    #[inline(always)]
    fn from(value: Thread<'a>) -> Self {
        Self(value)
    }
}

impl<'a> ImmutableThread<'a> {
    /// Creates a thread value from a raw Vm and index on `vm` stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm to link to.
    /// * `index`: the index on the lua stack.
    ///
    /// returns: Table
    ///
    /// # Safety
    ///
    /// Must ensure that index points to a thread value and is absolute. If index is not absolute
    /// then using the produced thread value is UB. If the index points to any other type then
    /// using the produced thread value is also UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self(Thread::from_raw(vm, index))
    }

    /// Returns the absolute index of this table on the Lua stack.
    #[inline(always)]
    pub fn index(&self) -> i32 {
        self.0.index
    }

    /// Returns the thread stack object attached to this thread value.
    #[inline(always)]
    pub fn as_thread(&self) -> &core::Thread<'a> {
        &self.0.thread
    }
}

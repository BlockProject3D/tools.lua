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

use crate::ffi::lua::{lua_pushlightuserdata, lua_touserdata};
use crate::util::core::SimpleDrop;
use crate::vm::value::{ImmutableValue, IntoLua};
use crate::vm::Vm;

#[derive(Debug)]
pub struct RawPtr<T>(*const T);

unsafe impl<T> SimpleDrop for RawPtr<T> {}

impl<T> Clone for RawPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for RawPtr<T> {}

impl<T> RawPtr<T> {
    #[inline(always)]
    pub fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    /// Returns the raw underlying pointer.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    /// Returns the raw underlying pointer.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Extracts a [RawPtr] from the given Lua [Vm] index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] where to read the lightuserdata from.
    /// * `index`: the index of the lightuserdata pointer on the stack.
    ///
    /// # Safety
    ///
    /// Calling this function assumes the given index is valid for [Vm] and the lightuserdata
    /// pointer points to an instance of T. If any of these assumptions are not respected,
    /// this function is UB.
    #[inline(always)]
    pub unsafe fn from_lua(vm: &Vm, index: i32) -> Self {
        Self(lua_touserdata(vm.as_ptr(), index) as _)
    }
}

unsafe impl<T> IntoLua for RawPtr<T> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlightuserdata(vm.as_ptr(), self.0 as _) };
        1
    }
}

unsafe impl<T: ImmutableValue> ImmutableValue for RawPtr<T> {}

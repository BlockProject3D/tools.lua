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
use crate::vm::registry::{FromIndex, Set};
use crate::vm::value::IntoLua;
use crate::vm::Vm;

#[derive(Debug)]
pub struct RawPtr<T>(*const T);

unsafe impl<T> SimpleDrop for RawPtr<T> { }

impl<T> Clone for RawPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for RawPtr<T> { }

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
}

unsafe impl<T> IntoLua for RawPtr<T> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) -> u16 {
        unsafe { lua_pushlightuserdata(vm.as_ptr(), self.0 as _) };
        1
    }
}

pub struct RawPtrRef<'a, T> {
    vm: &'a Vm,
    index: i32,
    ptr: RawPtr<T>,
}

impl<'a, T> RawPtrRef<'a, T> {
    /// Creates a [RawPtrRef] from a [Vm] and an index on the vm.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to attach to.
    /// * `index`: the index on the given [Vm] instance.
    ///
    /// returns: RawPtrRef<<unknown>>
    ///
    /// # Safety
    ///
    /// This function assumes that `index` points to a valid light-userdata object of type `T` on
    /// the stack represented by `vm`. Breaking any of these assumptions is UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self { vm, index, ptr: RawPtr::new(lua_touserdata(vm.as_ptr(), index) as _) }
    }

    pub fn from_ptr(vm: &'a Vm, ptr: RawPtr<T>) -> Self {
        ptr.into_lua(vm);
        Self { vm, index: vm.top(), ptr }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> RawPtr<T> {
        self.ptr
    }
}

impl<T: 'static> crate::vm::registry::Value for crate::vm::registry::types::RawPtr<T> {
    type Value<'a> = RawPtrRef<'a, T>;

    #[inline(always)]
    unsafe fn from_registry(vm: &Vm, index: i32) -> Self::Value<'_> {
        RawPtrRef::from_raw(vm, index)
    }

    #[inline(always)]
    fn push_registry<R: FromIndex>(value: Self::Value<'_>) -> R {
        unsafe { R::from_index(value.vm, value.index) }
    }

    #[inline(always)]
    unsafe fn set_registry(key: &impl Set, value: Self::Value<'_>) {
        key.set(value.vm, value.index);
    }
}

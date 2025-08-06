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

use std::marker::PhantomData;
use crate::ffi::lua::{lua_replace, lua_settop};
use crate::vm::registry::{FromIndex, Set};
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;

/// Represents a simple registry value.
///
/// # Safety
///
/// This trait always assumes a single lua value is managed. If the underlying type manipulates
/// multiple stack indices on the given lua [Vm], the implementation is considered UB.
pub unsafe trait SimpleRegistryValue {
    fn into_lua(self, vm: &Vm);

    /// Extracts an instance of `Self` from the given Lua [Vm] and index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] where to read the object from.
    /// * `index`: the index of the object on the stack.
    ///
    /// returns: Self
    ///
    /// # Safety
    ///
    /// This function assumes the given index is valid for [Vm] and that the object on the stack at
    /// the given index is already of type `Self`. If any of these assumptions are broken, this
    /// function is UB.
    unsafe fn from_lua(vm: &Vm, index: i32) -> Self;
}

pub struct LuaRef<'a, T> {
    vm: &'a Vm,
    index: i32,
    useless: PhantomData<T>,
}

impl<'a, T: SimpleRegistryValue> LuaRef<'a, T> {
    pub fn new(vm: &'a Vm, value: T) -> Self {
        value.into_lua(vm);
        Self {
            vm,
            index: vm.top(),
            useless: PhantomData,
        }
    }

    /// Constructs a [LuaRef] from a raw index on a Lua [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] where to read the lightuserdata from.
    /// * `index`: the index of the lightuserdata pointer on the stack.
    ///
    /// # Safety
    ///
    /// Calling this function assumes the given index is absolute and is valid for [Vm] and that
    /// the object on the stack at the given index is already of type `T`. If any of
    /// these assumptions are not respected, this function is UB.
    #[inline(always)]
    pub unsafe fn from_raw(vm: &'a Vm, index: i32) -> Self {
        Self {
            vm,
            index,
            useless: PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self) -> T {
        unsafe { T::from_lua(self.vm, self.index) }
    }

    pub fn set(&self, value: T) {
        value.into_lua(self.vm);
        unsafe { lua_replace(self.vm.as_ptr(), self.index) };
    }
}

impl<'a, T> Drop for LuaRef<'a, T> {
    fn drop(&mut self) {
        // Remove the object from the lua stack if it is on top of the stack.
        if self.index == self.vm.top() {
            unsafe { lua_settop(self.vm.as_ptr(), -2); }
        }
    }
}

impl<T: SimpleRegistryValue + 'static> super::Value for super::types::LuaRef<T> {
    type Value<'a> = LuaRef<'a, T>;

    unsafe fn from_registry(vm: &Vm, index: i32) -> Self::Value<'_> {
        LuaRef::from_raw(vm, vm.get_absolute_index(index))
    }

    fn push_registry<R: FromIndex>(value: Self::Value<'_>) -> R {
        unsafe { R::from_index(value.vm, value.index) }
    }

    unsafe fn set_registry(key: &impl Set, value: Self::Value<'_>) {
        key.set(value.vm, value.index);
    }
}

unsafe impl<T> SimpleRegistryValue for T where for<'a> T: FromLua<'a> + IntoLua {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) {
        IntoLua::into_lua(self, vm);
    }

    #[inline(always)]
    unsafe fn from_lua(vm: &Vm, index: i32) -> Self {
        unsafe { FromLua::from_lua_unchecked(vm, index) }
    }
}

unsafe impl<T> SimpleRegistryValue for RawPtr<T> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) {
        IntoLua::into_lua(self, vm);
    }

    #[inline(always)]
    unsafe fn from_lua(vm: &Vm, index: i32) -> Self {
        unsafe { RawPtr::from_lua(vm, index) }
    }
}

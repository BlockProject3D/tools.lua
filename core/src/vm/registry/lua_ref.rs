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
use crate::impl_simple_registry_value_static;
use crate::vm::registry::{FromIndex, Set};
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;

/// Represents a simple value type which can be manipulated by [LuaRef].
///
/// # Notes
///
/// * The definition of a simple type in bp3d-lua is a type which does not hold a reference to
/// the [Vm]. This is typically the case of primitives like strings, integers, numbers, etc. Types
/// such as tables or functions are called complex in bp3d-lua as they require constant interactions
/// with the Lua stack represented by a [Vm] in order to operate on them.
///
/// * For complex types, no wrapper is needed as they already have a reference to the attached [Vm]
/// in their value type. Instead, complex types which can be saved in the registry directly
/// implements the registry [Value](crate::vm::registry::Value) trait.
///
/// # Safety
///
/// This trait always assumes a single lua value is managed. If the underlying [SimpleValue]
/// manipulates multiple stack indices on the given lua [Vm], the implementation is considered UB.
pub unsafe trait SimpleValue<'a> {
    fn into_lua(self, vm: &'a Vm);

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
    unsafe fn from_lua(vm: &'a Vm, index: i32) -> Self;
}

/// Marks a type as being compatible with the [LuaRef](crate::vm::registry::types::LuaRef) based
/// registry system for simple types.
pub trait SimpleRegistryValue {
    type Value<'a>: SimpleValue<'a>;
}

pub struct LuaRef<'a, T> {
    vm: &'a Vm,
    index: i32,
    useless: PhantomData<T>,
}

impl<'a, T: SimpleValue<'a>> LuaRef<'a, T> {
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

    pub fn set(&mut self, value: T) {
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
    type Value<'a> = LuaRef<'a, T::Value<'a>>;

    unsafe fn from_registry(vm: &Vm, index: i32) -> Self::Value<'_> {
        LuaRef::from_raw(vm, vm.get_absolute_index(index))
    }

    fn push_registry<R: FromIndex>(value: Self::Value<'_>) -> R {
        unsafe {
            let r = R::from_index(value.vm, value.index);
            // Avoid calling the destructor which may try to pop an already popped value.
            std::mem::forget(value);
            r
        }
    }

    unsafe fn set_registry(key: &impl Set, value: Self::Value<'_>) {
        key.set(value.vm, value.index);
        // Avoid calling the destructor which may try to pop an already popped value.
        std::mem::forget(value);
    }
}

unsafe impl<'a, T> SimpleValue<'a> for T where T: FromLua<'a> + IntoLua {
    #[inline(always)]
    fn into_lua(self, vm: &'a Vm) {
        // This ensures the safety guarentee still holds.
        assert_eq!(IntoLua::into_lua(self, vm), 1);
    }

    #[inline(always)]
    unsafe fn from_lua(vm: &'a Vm, index: i32) -> Self {
        T::from_lua_unchecked(vm, index)
    }
}

unsafe impl<T> SimpleValue<'_> for RawPtr<T> {
    #[inline(always)]
    fn into_lua(self, vm: &Vm) {
        IntoLua::into_lua(self, vm);
    }

    #[inline(always)]
    unsafe fn from_lua(vm: &Vm, index: i32) -> Self {
        unsafe { RawPtr::from_lua(vm, index) }
    }
}

impl_simple_registry_value_static! {
    <T> (RawPtr<T>) => RawPtr<T>;
    (&str) => &'a str;
    (f32) => f32;
    (f64) => f64;
    (i8) => i8;
    (i16) => i16;
    (i32) => i32;
    (i64) => i64;
    (u8) => u8;
    (u16) => u16;
    (u32) => u32;
    (u64) => u64;
    (String) => String;
    (&[u8]) => &'a [u8];
    (Box<[u8]>) => Box<[u8]>;
}

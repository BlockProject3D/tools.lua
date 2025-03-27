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

use std::ffi::c_int;
use std::marker::PhantomData;
use crate::ffi::laux::{luaL_ref, luaL_unref};
use crate::ffi::lua::{lua_rawgeti, lua_rawseti, REGISTRYINDEX};
use crate::vm::registry::RegistryValue;
use crate::vm::Vm;

//TODO: Check if key can be a NonZeroI32

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct RawRegistryKey(c_int);

impl RawRegistryKey {
    /// Returns the raw key.
    #[inline(always)]
    pub fn as_int(&self) -> c_int {
        self.0
    }

    /// Wraps a raw integer as a registry key.
    #[inline(always)]
    pub fn from_int(v: c_int) -> RawRegistryKey {
        RawRegistryKey(v)
    }

    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    /// 
    /// # Safety
    /// 
    /// This is UB to call if the key is invalid or already freed.
    #[inline(always)]
    pub unsafe fn push(&self, vm: &Vm) {
        lua_rawgeti(vm.as_ptr(), REGISTRYINDEX, self.0);
    }

    /// Deletes this registry key from the specified [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to unregister from.
    ///
    /// returns: ()
    /// 
    /// # Safety
    ///
    /// This is UB to call if the registry key is invalid or was already freed.
    #[inline(always)]
    pub unsafe fn delete(self, vm: &Vm) {
        luaL_unref(vm.as_ptr(), REGISTRYINDEX, self.0);
    }

    /// Replaces the content of this key with the value on top of the stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm associated with this key.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// This is UB to call if the key has already been deleted.
    #[inline(always)]
    pub unsafe fn replace(&self, vm: &Vm) {
        lua_rawseti(vm.as_ptr(), REGISTRYINDEX, self.0);
    }

    /// Creates a new [RawRegistryKey] from the top of the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance representing the lua stack.
    ///
    /// returns: RegistryKey<T>
    ///
    /// # Safety
    /// 
    /// This is UB to call if the stack is empty.
    #[inline(always)]
    pub unsafe fn from_top(vm: &Vm) -> RawRegistryKey {
        let key = unsafe { luaL_ref(vm.as_ptr(), REGISTRYINDEX) };
        RawRegistryKey(key)
    }
}

pub struct RegistryKey<T> {
    raw: RawRegistryKey,
    useless: PhantomData<T>
}

impl<T: RegistryValue> RegistryKey<T> {
    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    #[inline(always)]
    pub fn push<'a>(&self, vm: &'a Vm) -> T::Value<'a> {
        unsafe { self.raw.push(vm) };
        unsafe { T::to_lua_value(vm, -1) }
    }

    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    #[inline(always)]
    pub fn as_raw(&self) -> RawRegistryKey {
        self.raw
    }

    /// Deletes this registry key from the specified [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to unregister from.
    ///
    /// returns: ()
    #[inline(always)]
    pub fn delete(self, vm: &Vm) {
        unsafe { self.raw.delete(vm) };
    }

    /// Creates a new [RegistryKey] from the top of the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance representing the lua stack.
    ///
    /// returns: RegistryKey<T>
    ///
    /// # Safety
    ///
    /// The type T must match the type of the value at the top of the stack. Additionally, the value
    /// at the top of the stack must not be referenced as it will be popped.
    pub unsafe fn from_top(vm: &Vm) -> RegistryKey<T> {
        RegistryKey {
            raw: RawRegistryKey::from_top(vm),
            useless: PhantomData
        }
    }
}

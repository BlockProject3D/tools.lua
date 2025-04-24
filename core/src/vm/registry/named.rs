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

use std::ffi::c_void;
use std::marker::PhantomData;
use crate::ffi::lua::{lua_insert, lua_pushlightuserdata, lua_pushnil, lua_rawget, lua_rawset, REGISTRYINDEX};
use crate::vm::registry::{Set, Value};
use crate::vm::value::util::ensure_value_top;
use crate::vm::Vm;

#[derive(Debug, Copy, Clone)]
pub struct RawKey(*const c_void);

impl RawKey {
    /// Pushes the value associated with this key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to manipulate.
    ///
    /// returns: ()
    pub fn push(&self, vm: &Vm) {
        let l = vm.as_ptr();
        unsafe {
            lua_pushlightuserdata(l, self.0 as _);
            lua_rawget(l, REGISTRYINDEX);
        }
    }

    pub const fn new(name: &str) -> Self {
        // This is a re-write of https://github.com/BPXFormat/bpx-rs/blob/develop/src/hash.rs
        // in const context.
        let mut val: u64 = 5381;
        let bytes = name.as_bytes();
        // Unreadable algorithm: see the hash loop in bpx-rs for the readable variant.
        let mut i = 0;
        while i != bytes.len() {
            let temp1 = val.wrapping_shl(5);
            let temp2 = temp1.wrapping_add(val);
            val = temp2.wrapping_add(bytes[i] as u64);
            i += 1;
        }
        // And now a hack to turn u64 into ptr (btw, do NOT dereference it).
        Self(val as usize as *const c_void)
    }
}

impl Set for RawKey {
    unsafe fn set(&self, vm: &Vm, index: i32) {
        let l = vm.as_ptr();
        ensure_value_top(vm, index);
        lua_pushlightuserdata(l, self.0 as _);
        lua_insert(l, -2); // Move key after value;
        lua_rawset(l, REGISTRYINDEX);
    }
}

pub struct Key<T> {
    raw: RawKey,
    useless: PhantomData<*const T>,
}

impl<T: Value> Key<T> {
    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    #[inline(always)]
    pub fn push<'a>(&self, vm: &'a Vm) -> T::Value<'a> {
        self.raw.push(vm);
        unsafe { T::from_registry(vm, -1) }
    }

    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    #[inline(always)]
    pub fn as_raw(&self) -> RawKey {
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
        unsafe {
            lua_pushnil(vm.as_ptr());
            self.raw.set(vm, -1);
        }
    }

    /// Sets the value for this key.
    ///
    /// # Arguments
    ///
    /// * `value`: the value to replace with.
    ///
    /// returns: ()
    pub fn set(&self, value: T::Value<'_>) {
        unsafe { T::set_registry(&self.raw, value) };
    }

    pub fn new(raw_key: RawKey, value: T::Value<'_>) -> Self {
        let key = Key { raw: raw_key, useless: PhantomData };
        key.set(value);
        key
    }
}

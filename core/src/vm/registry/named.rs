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

use std::cell::Cell;
use std::ffi::c_void;
use std::marker::PhantomData;
use crate::ffi::lua::{lua_createtable, lua_insert, lua_pushboolean, lua_pushlightuserdata, lua_pushnil, lua_rawget, lua_rawset, lua_settop, lua_type, State, Type, REGISTRYINDEX};
use crate::vm::registry::{Set, Value};
use crate::vm::value::util::ensure_value_top;
use crate::vm::Vm;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RawKey(*const c_void);

unsafe impl Send for RawKey {}
unsafe impl Sync for RawKey {}

impl RawKey {
    /// Pushes the value associated with this key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to manipulate.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// This is UB to call if the key was not registered in the given [Vm] using
    /// [register](RawKey::register).
    pub unsafe fn push(&self, vm: &Vm) {
        let l = vm.as_ptr();
        unsafe {
            lua_pushlightuserdata(l, self.0 as _);
            lua_rawget(l, REGISTRYINDEX);
        }
    }

    /// Attempts to register this key with the given [Vm] instance. This function ensures the key
    /// does not collide.
    ///
    /// # Panic
    ///
    /// This function panics if this key is already registered in the given [Vm].
    #[inline(always)]
    pub fn register(&self, vm: &Vm) {
        unsafe { check_key_already_used(vm, self.0) };
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

unsafe fn rawsetp(l: State, idx: i32, key: *const c_void) {
    lua_pushlightuserdata(l, key as _);
    lua_insert(l, -2); // Move key after value;
    lua_rawset(l, idx);
}

impl Set for RawKey {
    unsafe fn set(&self, vm: &Vm, index: i32) {
        let l = vm.as_ptr();
        ensure_value_top(vm, index);
        rawsetp(l, REGISTRYINDEX, self.0);
    }
}

struct InitKey(*const c_void);

const USED_KEYS: RawKey = RawKey::new("__used_keys__");

unsafe fn check_key_already_used(vm: &Vm, key: *const c_void) {
    if key == USED_KEYS.0 {
        panic!("Attempt to use reserved named key __used_keys__");
    }
    let l = vm.as_ptr();
    USED_KEYS.push(vm);
    if lua_type(l, -1) != Type::Table {
        lua_settop(l, -2); // Clear nil from the top of the stack.
        lua_createtable(l, 0, 0);
        USED_KEYS.set(vm, -1); // Pop the table and set it in the registry.
        USED_KEYS.push(vm); // Re-push the table so that following code can use it.
    }
    lua_pushlightuserdata(l, key as _);
    lua_rawget(l, -2); // Table is now at index -2 on the stack.
    let ty = lua_type(l, -1);
    lua_settop(l, -2); // Remove value from stack.
    if ty == Type::Boolean {
        // Key is already taken, this is bad.
        panic!("Attempt to register an already used named key");
    }
    lua_pushlightuserdata(l, key as _);
    lua_pushboolean(l, 1);
    lua_rawset(l, -3); // Table is now at -3 on the stack because we've just pushed a key and a
    // value.
    lua_settop(l, -2); // Clear the used keys table from the stack.
}

impl Set for InitKey {
    unsafe fn set(&self, vm: &Vm, index: i32) {
        check_key_already_used(vm, self.0);
        let l = vm.as_ptr();
        ensure_value_top(vm, index);
        rawsetp(l, REGISTRYINDEX, self.0);
    }
}

pub struct Key<T> {
    raw: RawKey,
    registered: Cell<bool>,
    useless: PhantomData<*const T>,
}

impl<T: Value> Key<T> {
    #[inline(always)]
    fn ensure_registered(&self, vm: &Vm) {
        if !self.registered.get() {
            unsafe { check_key_already_used(vm, self.raw.0) };
            self.registered.set(true);
        }
    }

    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    pub fn push<'a>(&self, vm: &'a Vm) -> Option<T::Value<'a>> {
        self.ensure_registered(vm);
        unsafe {
            self.raw.push(vm);
            if lua_type(vm.as_ptr(), -1) == Type::Nil {
                lua_settop(vm.as_ptr(), -2);
                return None;
            }
            Some(T::from_registry(vm, -1))
        }
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

    /// Resets the value pointed to by this registry key from the specified [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to unregister from.
    ///
    /// returns: ()
    #[inline(always)]
    pub fn reset(&self, vm: &Vm) {
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
        if self.registered.get() {
            unsafe { T::set_registry(&InitKey(self.raw.0), value) };
        } else {
            unsafe { T::set_registry(&self.raw, value) };
        }
    }

    pub const fn new(name: &str) -> Self {
        Key {
            raw: RawKey::new(name),
            registered: Cell::new(false),
            useless: PhantomData
        }
    }
}

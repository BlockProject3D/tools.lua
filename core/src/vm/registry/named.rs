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

use std::collections::BTreeSet;
use crate::ffi::lua::{lua_insert, lua_pushlightuserdata, lua_rawget, lua_rawset, State, Type, REGISTRYINDEX};
use crate::vm::registry::{Set, Value};
use crate::vm::value::util::{ensure_type_equals, ensure_value_top};
use crate::vm::Vm;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use bp3d_debug::debug;
use crate::ffi::ext::{lua_ext_keyreg_get, lua_ext_keyreg_ref, lua_ext_keyreg_set, lua_ext_keyreg_unref};

#[derive(Debug)]
pub struct RawKey {
    ptr: *const c_void,
    registered: AtomicBool,
    register_lock: Mutex<bool>
}

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
    pub fn push(&self, vm: &Vm) {
        check_register_key_unique(self);
        let l = vm.as_ptr();
        unsafe {
            lua_pushlightuserdata(l, self.ptr as _);
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
        Self {
            ptr: val as usize as *const c_void,
            registered: AtomicBool::new(false),
            register_lock: Mutex::new(false)
        }
    }
}

unsafe fn rawsetp(l: State, idx: i32, key: *const c_void) {
    lua_pushlightuserdata(l, key as _);
    lua_insert(l, -2); // Move key after value;
    lua_rawset(l, idx);
}

impl Set for RawKey {
    unsafe fn set(&self, vm: &Vm, index: i32) {
        check_register_key_unique(self);
        let l = vm.as_ptr();
        ensure_value_top(vm, index);
        rawsetp(l, REGISTRYINDEX, self.ptr);
    }
}

fn check_register_key_unique(key: &RawKey) {
    if key.registered.load(Ordering::Relaxed) {
        return;
    }
    let mut registered = key.register_lock.lock().unwrap();
    if *registered {
        return;
    }
    let registry = unsafe { voidp_to_ref(lua_ext_keyreg_get()) };
    let mut lock = registry.lock().unwrap();
    if lock.contains(&(key.ptr as _)) {
        panic!("Attempt to register a duplicate key");
    } else {
        lock.insert(key.ptr as _);
        *registered = true;
        key.registered.store(true, Ordering::Relaxed);
    }
}

unsafe fn voidp_to_ref(p: *const c_void) -> &'static Mutex<BTreeSet<usize>>
{
    assert!(!p.is_null());
    unsafe { &*(p as *const Mutex<BTreeSet<usize>>) }
}

unsafe fn voidp_to_ptr(p: *const c_void) -> *mut Mutex<BTreeSet<usize>>
{
    assert!(!p.is_null());
    p as *mut Mutex<BTreeSet<usize>>
}

fn ref_to_voidp(r: &'static Mutex<BTreeSet<usize>>) -> *const c_void
{
    r as *const Mutex<BTreeSet<usize>> as *const c_void
}

pub(crate) fn handle_root_vm_init() {
    let refs = unsafe { lua_ext_keyreg_ref() };
    debug!({refs}, "Init RootVM");
    if refs == 1 { // First reference, initialize the key registry...
        debug!("Setting up new named key registry...");
        let ptr = ref_to_voidp(Box::leak(Box::new(Mutex::new(BTreeSet::new()))));
        unsafe { lua_ext_keyreg_set(ptr) };
    }
}

pub(crate) fn handle_root_vm_uninit() {
    let refs = unsafe { lua_ext_keyreg_unref() };
    if refs == 0 {
        debug!("Closing named key registry...");
        unsafe {
            drop(Box::from_raw(voidp_to_ptr(lua_ext_keyreg_get())));
            lua_ext_keyreg_set(std::ptr::null_mut());
        }
    }
}

pub struct Key<T> {
    raw: RawKey,
    useless: PhantomData<*const T>
}

unsafe impl<T> Send for Key<T> {}
unsafe impl<T> Sync for Key<T> {}

impl<T: Value> Key<T> {
    /// Pushes the lua value associated to this registry key on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to attach the produced lua value to.
    ///
    /// returns: <T as RegistryValue>::Value
    pub fn push<'a>(&self, vm: &'a Vm) -> Option<T::Value<'a>> {
        unsafe {
            self.raw.push(vm);
            ensure_type_equals(vm, -1, Type::LightUserdata)
                .map(|_| T::from_registry(vm, -1)).ok()
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
    pub fn as_raw(&self) -> &RawKey {
        &self.raw
    }

    #[inline(always)]
    pub const fn new(name: &str) -> Key<T> {
        Key {
            raw: RawKey::new(name),
            useless: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set(&self, value: T::Value<'_>) {
        unsafe { T::set_registry(&self.raw, value) }
    }
}

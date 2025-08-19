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

use std::collections::HashMap;
use crate::ffi::lua::{lua_insert, lua_pushlightuserdata, lua_rawget, lua_rawset, lua_settop, lua_type, State, Type, REGISTRYINDEX};
use crate::vm::registry::{Set, Value};
use crate::vm::value::util::move_value_top;
use crate::vm::Vm;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use bp3d_debug::debug;
use crate::ffi::ext::{lua_ext_keyreg_get, lua_ext_keyreg_ref, lua_ext_keyreg_unref};

#[derive(Debug)]
pub struct RawKey {
    ptr: *const c_void,
    // This may not always work, but unfortunately Rust TypeId is broken across modules.
    // Fortunately, the generic type which is used with this is always a static lifetime
    // which must implement the Value trait which limits the number of possible types.
    ty: fn() -> &'static str,
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

    pub fn ty(&self) -> &'static str {
        (self.ty)()
    }

    pub const fn new(name: &str, ty: fn() -> &'static str) -> Self {
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
            ty,
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
        move_value_top(vm, index);
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
    if let Some(ty) = lock.get(&(key.ptr as _)) {
        if *ty != key.ty() {
            panic!("Attempt to register a duplicate key");
        }
    }
    lock.insert(key.ptr as _, key.ty());
    *registered = true;
    key.registered.store(true, Ordering::Relaxed);
}

type NamedKeyRegistry = Mutex<HashMap<usize, &'static str>>;

unsafe fn voidp_to_ref(p: *mut c_void) -> &'static NamedKeyRegistry
{
    assert!(!p.is_null());
    unsafe { &*(p as *const NamedKeyRegistry) }
}

unsafe fn voidp_to_ptr(p: *mut c_void) -> *mut NamedKeyRegistry
{
    assert!(!p.is_null());
    p as *mut NamedKeyRegistry
}

fn ref_to_voidp(r: &'static NamedKeyRegistry) -> *mut c_void
{
    r as *const NamedKeyRegistry as *mut c_void
}

pub(crate) fn handle_root_vm_init() {
    let ptr = ref_to_voidp(Box::leak(Box::new(Mutex::new(HashMap::new()))));
    // Pointer set in lua_ext_keyreg_ref to avoid TOCTOU.
    let ptr = unsafe { lua_ext_keyreg_ref(ptr) };
    if ptr.is_null() {
        debug!("Set up new named key registry...");
    } else {
        debug!("Named key registry already exists");
        unsafe { drop(Box::from_raw(voidp_to_ptr(ptr))) };
    }
}

pub(crate) fn handle_root_vm_uninit() {
    // Pointer reset to NULL in lua_ext_keyreg_unref to avoid TOCTOU.
    let ptr = unsafe { lua_ext_keyreg_unref() };
    if !ptr.is_null() {
        debug!("Closing named key registry...");
        unsafe { drop(Box::from_raw(voidp_to_ptr(ptr))) };
    } else {
        debug!("Named key registry is still in use");
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
            if lua_type(vm.as_ptr(), -1) == Type::Nil {
                lua_settop(vm.as_ptr(), -2); // Pop the nil from the stack.
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
    pub fn as_raw(&self) -> &RawKey {
        &self.raw
    }

    #[inline(always)]
    pub const fn new(name: &str) -> Key<T> {
        Key {
            raw: RawKey::new(name, std::any::type_name::<T>),
            useless: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set(&self, value: T::Value<'_>) {
        unsafe { T::set_registry(&self.raw, value) }
    }
}

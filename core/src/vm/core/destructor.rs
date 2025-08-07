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

use crate::vm::registry::named::Key;
use crate::vm::Vm;
use bp3d_debug::debug;
use std::sync::Arc;
use crate::vm::registry::types::LuaRef;
use crate::vm::value::types::RawPtr;
use crate::vm::registry::lua_ref::LuaRef as LiveLuaRef;

/// This trait represents a value which can be attached to a [Pool](Pool).
pub trait RawSend: Send {
    type Ptr: Copy;

    fn into_raw(self) -> Self::Ptr;

    /// Deletes the raw pointer.
    ///
    /// # Safety
    ///
    /// This function must be called with the same pointer that originated from the same type using
    /// the [into_raw](Raw::into_raw) method.
    unsafe fn delete(ptr: Self::Ptr);
}

/// This trait represents a value which can be attached to a [Pool](Pool).
pub trait Raw {
    type Ptr: Copy;

    fn into_raw(self) -> Self::Ptr;

    /// Deletes the raw pointer.
    ///
    /// # Safety
    ///
    /// This function must be called with the same pointer that originated from the same type using
    /// the [into_raw](Raw::into_raw) method.
    unsafe fn delete(ptr: Self::Ptr);
}

impl<T: Raw + Send> RawSend for T {
    type Ptr = T::Ptr;

    fn into_raw(self) -> Self::Ptr {
        T::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        T::delete(ptr)
    }
}

impl<T> Raw for Box<T> {
    type Ptr = *mut T;

    fn into_raw(self) -> Self::Ptr {
        Box::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        drop(Box::from_raw(ptr))
    }
}

impl<T> Raw for std::rc::Rc<T> {
    type Ptr = *const T;

    fn into_raw(self) -> Self::Ptr {
        std::rc::Rc::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        drop(std::rc::Rc::from_raw(ptr))
    }
}

impl<T: Send + Sync> RawSend for Arc<T> {
    type Ptr = *const T;

    fn into_raw(self) -> Self::Ptr {
        Arc::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        drop(Arc::from_raw(ptr))
    }
}

static DESTRUCTOR_POOL: Key<LuaRef<RawPtr<Pool>>> = Key::new("__destructor_pool__");

pub struct Pool {
    leaked: Vec<Box<dyn FnOnce()>>,
    is_send: bool
}

impl Pool {
    pub fn new(is_send: bool) -> Self {
        Self {
            leaked: Vec::new(),
            is_send
        }
    }

    /// Inserts this pool in the given Vm.
    ///
    /// # Safety
    ///
    /// This is only safe to be called on [RootVm](crate::vm::RootVm) construction.
    pub unsafe fn new_in_vm(vm: &mut Vm, is_send: bool) {
        let b = Box::leak(Box::new(Pool::new(is_send)));
        let ptr = RawPtr::new(b as *mut Pool);
        DESTRUCTOR_POOL.set(LiveLuaRef::new(vm, ptr));
    }

    /// Extracts a destructor pool from the given [Vm].
    ///
    /// # Safety
    ///
    /// The returned reference must not be aliased.
    unsafe fn _from_vm(vm: &Vm) -> RawPtr<Self> {
        let ptr = DESTRUCTOR_POOL.push(vm).unwrap();
        ptr.get()
    }

    pub fn from_vm(vm: &mut Vm) -> &mut Self {
        unsafe { &mut *Self::_from_vm(vm).as_mut_ptr() }
    }

    pub fn attach_send<R: RawSend>(vm: &Vm, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        let ptr = unsafe { Self::_from_vm(vm) };
        unsafe { (*ptr.as_mut_ptr()).attach_mut_send(raw) }
    }

    pub fn attach<R: Raw>(vm: &Vm, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        let ptr = unsafe { Self::_from_vm(vm) };
        unsafe { (*ptr.as_mut_ptr()).attach_mut(raw) }
    }

    pub fn attach_mut_send<R: RawSend>(&mut self, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        let ptr = R::into_raw(raw);
        self.leaked.push(Box::new(move || {
            unsafe { R::delete(ptr) };
        }));
        ptr
    }

    pub fn attach_mut<R: Raw>(&mut self, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        if self.is_send {
            panic!("Attempt to attach !Send type to Send destructor Pool: this is forbidden!")
        }
        let ptr = R::into_raw(raw);
        self.leaked.push(Box::new(move || {
            unsafe { R::delete(ptr) };
        }));
        ptr
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        debug!({ num = self.leaked.len() }, "Deleting leaked pointers...");
        let v = std::mem::take(&mut self.leaked);
        for f in v {
            f()
        }
    }
}

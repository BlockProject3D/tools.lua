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

use crate::ffi::lua::{
    lua_gettable, lua_pushlightuserdata, lua_pushstring, lua_settable, lua_settop, lua_touserdata,
    REGISTRYINDEX,
};
use crate::vm::Vm;
use bp3d_debug::debug;
use std::rc::Rc;

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

impl<T> Raw for Box<T> {
    type Ptr = *mut T;

    fn into_raw(self) -> Self::Ptr {
        Box::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        drop(Box::from_raw(ptr))
    }
}

impl<T> Raw for Rc<T> {
    type Ptr = *const T;

    fn into_raw(self) -> Self::Ptr {
        Rc::into_raw(self)
    }

    unsafe fn delete(ptr: Self::Ptr) {
        drop(Rc::from_raw(ptr))
    }
}

#[derive(Default)]
pub struct Pool {
    leaked: Vec<Box<dyn FnOnce()>>,
}

impl Pool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts this pool in the given Vm.
    ///
    /// # Safety
    ///
    /// This is only safe to be called on [RootVm](crate::vm::RootVm) construction.
    pub unsafe fn new_in_vm(vm: &mut Vm) {
        let l = vm.as_ptr();
        let b = Box::leak(Box::new(Pool::new()));
        unsafe {
            lua_pushstring(l, c"__destructor_pool__".as_ptr());
            let ptr = b as *mut Pool as _;
            lua_pushlightuserdata(l, ptr);
            lua_settable(l, REGISTRYINDEX);
        };
    }

    /// Extracts a destructor pool from the given [Vm].
    ///
    /// # Safety
    ///
    /// The returned reference must not be aliased.
    unsafe fn _from_vm(vm: &Vm) -> *mut Self {
        let l = vm.as_ptr();
        unsafe {
            lua_pushstring(l, c"__destructor_pool__".as_ptr());
            lua_gettable(l, REGISTRYINDEX);
            let ptr = lua_touserdata(l, -1) as *mut Pool;
            assert!(!ptr.is_null());
            lua_settop(l, -2); // Remove the pointer from the lua stack.
            ptr
        }
    }

    pub fn from_vm(vm: &mut Vm) -> &mut Self {
        unsafe { &mut *Self::_from_vm(vm) }
    }

    pub fn attach<R: Raw>(vm: &Vm, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        let ptr = unsafe { Self::_from_vm(vm) };
        unsafe { (&mut *ptr).attach_mut(raw) }
    }

    pub fn attach_mut<R: Raw>(&mut self, raw: R) -> R::Ptr
    where
        R::Ptr: 'static,
    {
        let ptr = R::into_raw(raw);
        self.leaked.push(Box::new(move || {
            unsafe { R::delete(ptr) };
        }));
        ptr
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        debug!(
            { num = self.leaked.len() as u32 },
            "Deleting leaked pointers..."
        );
        let v = std::mem::take(&mut self.leaked);
        for f in v {
            f()
        }
    }
}

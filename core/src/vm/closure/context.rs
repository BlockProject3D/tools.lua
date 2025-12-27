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

//! Second version of the context tool.

use crate::ffi::laux::luaL_error;
use crate::ffi::lua::lua_newuserdata;
use crate::util::core::SimpleDrop;
use crate::vm::closure::{FromUpvalue, IntoUpvalue, Upvalue};
use crate::vm::registry::core::RawKey;
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct Cell<T> {
    ptr: *mut *const T,
}

#[cfg(feature = "send")]
impl<T: Send> Cell<T> {
    pub fn new(ctx: Context<T>) -> Self {
        Self { ptr: ctx.ptr }
    }
}

#[cfg(not(feature = "send"))]
impl<T> Cell<T> {
    pub fn new(ctx: Context<T>) -> Self {
        Self { ptr: ctx.ptr }
    }
}

impl<T> Cell<T> {
    pub fn bind<'a>(&mut self, obj: &'a T) -> Guard<'a, T> {
        unsafe { *self.ptr = obj as _ };
        Guard {
            useless: PhantomData,
            ud: self.ptr,
        }
    }
}

pub struct CellMut<T> {
    ptr: *mut *const T,
}

#[cfg(feature = "send")]
impl<T: Send> CellMut<T> {
    pub fn new(ctx: ContextMut<T>) -> Self {
        Self { ptr: ctx.0.ptr }
    }
}

#[cfg(not(feature = "send"))]
impl<T> CellMut<T> {
    pub fn new(ctx: ContextMut<T>) -> Self {
        Self { ptr: ctx.0.ptr }
    }
}

impl<T> CellMut<T> {
    pub fn bind<'a>(&mut self, obj: &'a mut T) -> Guard<'a, T> {
        unsafe { *self.ptr = obj as _ };
        Guard {
            useless: PhantomData,
            ud: self.ptr,
        }
    }
}

pub struct Context<T> {
    key: RawKey,
    ptr: *mut *const T,
}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Context<T> {}

pub struct ContextMut<T>(Context<T>);

impl<T> Clone for ContextMut<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ContextMut<T> {}

impl<T: 'static> Context<T> {
    pub fn new(vm: &Vm) -> Self {
        let (ptr, key) = unsafe {
            let ptr = lua_newuserdata(vm.as_ptr(), 8);
            std::ptr::write(ptr as *mut u64, 0);
            (ptr, RawKey::from_top(vm))
        };
        Self {
            key,
            ptr: ptr as *mut *const T,
        }
    }
}

impl<T: 'static> ContextMut<T> {
    pub fn new(vm: &Vm) -> Self {
        Self(Context::new(vm))
    }
}

#[repr(transparent)]
pub struct Guard<'a, T> {
    ud: *mut *const T,
    useless: PhantomData<&'a T>,
}

impl<T> Drop for Guard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            *self.ud = std::ptr::null();
        }
    }
}

#[repr(transparent)]
pub struct Ref<'a, T>(&'a T);

#[repr(transparent)]
pub struct Mut<'a, T>(&'a mut T);

impl<T: 'static> Deref for Ref<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: 'static> Deref for Mut<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: 'static> DerefMut for Mut<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

unsafe impl<T: 'static> SimpleDrop for Ref<'_, T> {}
unsafe impl<T: 'static> SimpleDrop for Mut<'_, T> {}

impl<'a, T: 'static> FromUpvalue<'a> for Ref<'a, T> {
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        let ptr: RawPtr<*const T> = FromUpvalue::from_upvalue(vm, index);
        if (*ptr.as_ptr()).is_null() {
            luaL_error(
                vm.as_ptr(),
                c"Context is not available in this function.".as_ptr(),
            );
            // luaL_error raises a lua exception and unwinds, so this cannot be reached.
            std::hint::unreachable_unchecked();
        }
        Ref(unsafe { &**ptr.as_ptr() })
    }
}

impl<'a, T: 'static> FromUpvalue<'a> for Mut<'a, T> {
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        let ptr: RawPtr<*mut T> = FromUpvalue::from_upvalue(vm, index);
        if (*ptr.as_ptr()).is_null() {
            luaL_error(
                vm.as_ptr(),
                c"Context is not available in this function.".as_ptr(),
            );
            // luaL_error raises a lua exception and unwinds, so this cannot be reached.
            std::hint::unreachable_unchecked();
        }
        Mut(unsafe { &mut **ptr.as_ptr() })
    }
}

impl<T: 'static> IntoUpvalue for Context<T> {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        unsafe { self.key.push(vm) };
        1
    }
}

impl<T: 'static> IntoUpvalue for ContextMut<T> {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        unsafe { self.0.key.push(vm) };
        1
    }
}

impl<T: 'static> Upvalue for Context<T> {
    type From<'a> = Ref<'a, T>;
}

impl<T: 'static> Upvalue for ContextMut<T> {
    type From<'a> = Mut<'a, T>;
}

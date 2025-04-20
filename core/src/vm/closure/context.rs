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

//! A module to simplify declaring functions with associated to a context (rust object).

//TODO: Investigate if wrapping the raw pointer in a userdata instead of a lightuserdata is any
// faster.

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use crate::ffi::laux::luaL_error;
use crate::ffi::lua::{lua_pushlightuserdata, lua_settop, lua_topointer};
use crate::util::SimpleDrop;
use crate::vm::closure::{FromUpvalue, IntoUpvalue, Upvalue};
use crate::vm::registry::core::{RawRegistryKey};
use crate::vm::Vm;

pub struct Context<T> {
    key: RawRegistryKey,
    useless: PhantomData<*const T>
}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            useless: self.useless
        }
    }
}

impl<T> Copy for Context<T> {}

pub struct ContextMut<T>(Context<T>);

impl<T> Clone for ContextMut<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for ContextMut<T> { }

impl<T: 'static> Context<T> {
    pub fn new(vm: &Vm) -> Self {
        let key = unsafe {
            lua_pushlightuserdata(vm.as_ptr(), std::ptr::null_mut());
            RawRegistryKey::from_top(vm)
        };
        Self {
            key,
            useless: PhantomData
        }
    }

    pub fn bind<'a, 'b>(&self, vm: &'a Vm, obj: &'b T) -> Guard<'a, &'b T> {
        unsafe {
            lua_pushlightuserdata(vm.as_ptr(), obj as *const T as *mut T as *mut c_void);
            self.key.replace(vm);
            Guard {
                vm,
                ptr: obj,
                key: self.key
            }
        }
    }
}

impl<T: 'static> ContextMut<T> {
    pub fn new(vm: &Vm) -> Self {
        Self(Context::new(vm))
    }

    pub fn bind<'a, 'b>(&self, vm: &'a Vm, obj: &'b mut T) -> Guard<'a, &'b mut T> {
        unsafe {
            lua_pushlightuserdata(vm.as_ptr(), obj as *mut T as *mut c_void);
            self.0.key.replace(vm);
            Guard {
                vm,
                ptr: obj,
                key: self.0.key
            }
        }
    }
}

pub struct Guard<'a, T> {
    vm: &'a Vm,
    #[allow(dead_code)]
    ptr: T,
    key: RawRegistryKey
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            lua_pushlightuserdata(self.vm.as_ptr(), std::ptr::null_mut());
            self.key.replace(self.vm);
        }
    }
}

#[repr(transparent)]
pub struct Ref<'a, T>(&'a T);

#[repr(transparent)]
pub struct Mut<'a, T>(&'a mut T);

impl<'a, T: 'static> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: 'static> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: 'static> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

unsafe impl<'a, T: 'static> SimpleDrop for Ref<'a, T> { }
unsafe impl<'a, T: 'static> SimpleDrop for Mut<'a, T> { }

impl<'a, T: 'static> FromUpvalue<'a> for Ref<'a, T> {
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        let key = RawRegistryKey::from_int(FromUpvalue::from_upvalue(vm, index));
        key.push(vm);
        let ptr = lua_topointer(vm.as_ptr(), -1) as *const T;
        //Remove lightuserdata on the top of the stack.
        lua_settop(vm.as_ptr(), -2);
        if ptr.is_null() {
            luaL_error(vm.as_ptr(), c"Context is not available in this function.".as_ptr());
            // luaL_error raises a lua exception and unwinds, so this cannot be reached.
            std::hint::unreachable_unchecked();
        }
        Ref(unsafe { &*ptr })
    }
}

impl<'a, T: 'static> FromUpvalue<'a> for Mut<'a, T> {
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        let key = RawRegistryKey::from_int(FromUpvalue::from_upvalue(vm, index));
        key.push(vm);
        let ptr = lua_topointer(vm.as_ptr(), -1) as *mut T;
        //Remove lightuserdata on the top of the stack.
        lua_settop(vm.as_ptr(), -2);
        if ptr.is_null() {
            luaL_error(vm.as_ptr(), c"Context is not available in this function.".as_ptr());
            // luaL_error raises a lua exception and unwinds, so this cannot be reached.
            std::hint::unreachable_unchecked();
        }
        Mut(unsafe { &mut *ptr })
    }
}

impl<T: 'static> IntoUpvalue for Context<T> {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.key.as_int().into_upvalue(vm)
    }
}

impl<T: 'static> IntoUpvalue for ContextMut<T> {
    fn into_upvalue(self, vm: &Vm) -> u16 {
        self.0.key.as_int().into_upvalue(vm)
    }
}

impl<T: 'static> Upvalue for Context<T> {
    type From<'a> = Ref<'a, T>;
}

impl<T: 'static> Upvalue for ContextMut<T> {
    type From<'a> = Mut<'a, T>;
}

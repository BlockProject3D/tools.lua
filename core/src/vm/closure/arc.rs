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

use crate::util::core::SimpleDrop;
use crate::vm::closure::{FromUpvalue, IntoUpvalue, Upvalue};
use crate::vm::Vm;
use std::ops::Deref;
use crate::vm::value::types::RawPtr;

pub type Shared<T> = std::sync::Arc<T>;

#[repr(transparent)]
pub struct Arc<T: Send + Sync>(*const T);

#[repr(transparent)]
pub struct Ref<'a, T: Send + Sync>(&'a T);

unsafe impl<T: Send + Sync> SimpleDrop for Ref<'_, T> {}

impl<T: Send + Sync> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: Send + Sync> FromUpvalue<'a> for Ref<'a, T> {
    #[inline(always)]
    unsafe fn from_upvalue(vm: &'a Vm, index: i32) -> Self {
        let ptr: RawPtr<T> = FromUpvalue::from_upvalue(vm, index);
        Ref(&*ptr.as_ptr())
    }
}

impl<T: Send + Sync + 'static> Upvalue for Arc<T> {
    type From<'a> = crate::vm::closure::rc::Ref<'a, T>;
}

impl<T: Send + Sync + 'static> IntoUpvalue for Arc<T> {
    #[inline(always)]
    fn into_upvalue(self, vm: &Vm) -> u16 {
        RawPtr::new(self.0 as *mut T).into_upvalue(vm)
    }
}

impl<T: Send + Sync + 'static> Arc<T> {
    #[inline(always)]
    pub fn from_rust(vm: &Vm, rc: Shared<T>) -> Arc<T> {
        Arc(crate::vm::core::destructor::Pool::attach_send(vm, rc))
    }
}

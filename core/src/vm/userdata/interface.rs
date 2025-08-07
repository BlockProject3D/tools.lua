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

use crate::vm::userdata::{core::Registry, Error};
use crate::vm::Vm;
use std::borrow::Cow;
use std::ffi::CStr;

/// This trait represents all types of UserData. An UserData is a type with a maximum alignment of 8
/// with its memory tied to the Lua GC.
#[cfg(feature = "send")]
pub trait UserData: Send + Sized {
    const CLASS_NAME: &'static CStr;

    fn register<C: NameConvert>(registry: &Registry<Self, C>) -> Result<(), Error>;
}

/// This trait represents all types of UserData. An UserData is a type with a maximum alignment of 8
/// with its memory tied to the Lua GC.
#[cfg(not(feature = "send"))]
pub trait UserData: Sized {
    const CLASS_NAME: &'static CStr;

    fn register<C: NameConvert>(registry: &Registry<Self, C>) -> Result<(), Error>;
}

/// This trait represents an UserData which is never borrowed mutably (excluding interior mutability
/// patterns).
///
/// # Safety
///
/// This is UB to implement on UserData types which may be borrowed mutably.
pub unsafe trait UserDataImmutable: UserData {}

pub trait LuaDrop {
    fn lua_drop(&self, vm: &Vm);
}

pub trait AddGcMethod<T: UserData> {
    fn add_gc_method<C: NameConvert>(&self, reg: &Registry<T, C>);
}

pub trait NameConvert {
    fn name_convert(&self, name: &'static CStr) -> Cow<'static, CStr>;
}

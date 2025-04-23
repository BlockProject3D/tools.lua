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

//! Core rust utilities module.

use std::borrow::Cow;
use std::ffi::{CStr, CString, OsStr};
use std::path::Path;

pub trait AnyStr {
    fn to_str(&self) -> crate::vm::Result<Cow<CStr>>;
}

impl AnyStr for &str {
    fn to_str(&self) -> crate::vm::Result<Cow<CStr>> {
        Ok(Cow::Owned(
            CString::new(&**self).map_err(|_| crate::vm::error::Error::Null)?,
        ))
    }
}

impl AnyStr for &CStr {
    #[inline(always)]
    fn to_str(&self) -> crate::vm::Result<Cow<CStr>> {
        Ok(Cow::Borrowed(&**self))
    }
}

/// Represents a type which can be trivially dropped (i.e. no Drop implementation).
///
/// # Safety
///
/// This is UB to implement this trait on types which are not trivially dropped.
pub unsafe trait SimpleDrop {}

unsafe impl<T> SimpleDrop for *mut T {}
unsafe impl<T> SimpleDrop for *const T {}
unsafe impl SimpleDrop for bool {}
unsafe impl SimpleDrop for &str {}
unsafe impl<T: SimpleDrop> SimpleDrop for Option<T> {}
unsafe impl<T: SimpleDrop, R: SimpleDrop> SimpleDrop for Result<T, R> {}
unsafe impl<T> SimpleDrop for &T {}
unsafe impl SimpleDrop for &[u8] {}
unsafe impl SimpleDrop for &OsStr {}
unsafe impl SimpleDrop for &Path {}

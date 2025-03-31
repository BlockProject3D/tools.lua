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

use std::fmt::Write;
use std::ffi::{CStr, CString};
use crate::ffi::laux::{luaL_loadbuffer, luaL_loadstring};
use crate::ffi::lua::{State, ThreadStatus};
use crate::vm::{Load, LoadString};
use crate::vm::core::util::ChunkNameBuilder;

impl LoadString for &CStr {
    #[inline(always)]
    fn load_string(&self, l: State) -> ThreadStatus {
        unsafe { luaL_loadstring(l, self.as_ptr()) }
    }
}

impl LoadString for &str {
    fn load_string(&self, l: State) -> ThreadStatus {
        let s = CString::new(*self);
        match s {
            Ok(v) => {
                (&*v).load_string(l)
            }
            Err(_) => ThreadStatus::ErrSyntax
        }
    }
}

pub struct Code<'a> {
    name: &'a str,
    code: &'a [u8]
}

impl<'a> Code<'a> {
    pub fn new(name: &'a str, code: &'a [u8]) -> Self {
        Self {
            name,
            code
        }
    }
}

impl Load for Code<'_> {
    fn load(&self, l: State) -> crate::vm::Result<ThreadStatus> {
        let mut builder = ChunkNameBuilder::new();
        let _ = write!(&mut builder, "={}", self.name);
        let name = builder.build();
        unsafe {
            Ok(luaL_loadbuffer(l, self.code.as_ptr() as _, self.code.len(), name.cstr().as_ptr()))
        }
    }
}

impl<T: LoadString> Load for T {
    fn load(&self, l: State) -> crate::vm::Result<ThreadStatus> {
        Ok(self.load_string(l))
    }
}

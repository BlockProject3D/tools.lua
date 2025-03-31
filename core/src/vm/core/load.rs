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
use std::ffi::{c_char, c_void, CStr, CString, OsStr};
use std::fs::File;
use std::path::Path;
use crate::ffi::laux::{luaL_loadbuffer, luaL_loadstring};
use crate::ffi::lua::{lua_load, State, ThreadStatus};
use crate::vm::core::{Load, LoadString};
use crate::vm::core::util::{ChunkName, ChunkNameBuilder};
use crate::vm::util::lua_rust_error;

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
    fn load(self, l: State) -> ThreadStatus {
        let mut builder = ChunkNameBuilder::new();
        let _ = write!(&mut builder, "={}", self.name);
        let name = builder.build();
        unsafe { luaL_loadbuffer(l, self.code.as_ptr() as _, self.code.len(), name.cstr().as_ptr()) }
    }
}

impl<T: LoadString> Load for T {
    fn load(self, l: State) -> ThreadStatus {
        self.load_string(l)
    }
}

pub trait Custom {
    type Error: std::error::Error;

    fn read_data(&mut self) -> Result<&[u8], Self::Error>;
}

/// Bind a custom Rust loader to Lua.
///
/// # Safety
///
/// This is UB to call outside a [Load] trait implementation.
pub unsafe fn load_custom<T: Custom>(l: State, chunk_name: ChunkName, mut custom: T) -> ThreadStatus {
    extern "C-unwind" fn _reader<T: Custom>(l: State, ud: *mut c_void, sz: *mut usize) -> *const c_char {
        let obj = ud as *mut T;
        unsafe {
            let res = (&mut *obj).read_data();
            match res {
                Err(e) => {
                    lua_rust_error(l, e);
                },
                Ok(v) => {
                    *sz = v.len();
                    v.as_ptr() as _
                }
            }
        }
    }
    lua_load(l, _reader::<T>, &mut custom as *mut T as _, chunk_name.cstr().as_ptr())
}

const BUF_SIZE: usize = 8192;

pub struct Read<T: std::io::Read> {
    inner: T,
    buffer: [u8; BUF_SIZE],
    len: usize
}

impl<T: std::io::Read> Read<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            buffer: [0; BUF_SIZE],
            len: 0
        }
    }
}

impl<T: std::io::Read> Custom for Read<T> {
    type Error = std::io::Error;

    fn read_data(&mut self) -> Result<&[u8], Self::Error> {
        self.len = self.inner.read(&mut self.buffer[..])?;
        Ok(&self.buffer[..self.len])
    }
}

pub struct Script {
    file: File,
    chunk_name: ChunkName
}

impl Script {
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut builder = ChunkNameBuilder::new();
        let file_name = path.as_ref().file_name().unwrap_or(OsStr::new("unnamed")).to_str().unwrap_or("not-unicode");
        let _ = write!(&mut builder, "@{}", file_name);
        let file = File::open(path)?;
        Ok(Self {
            file,
            chunk_name: builder.build(),
        })
    }
}

impl Load for Script {
    fn load(self, l: State) -> ThreadStatus {
        unsafe { load_custom(l, self.chunk_name, Read::new(self.file)) }
    }
}

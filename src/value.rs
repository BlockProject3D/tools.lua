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

use std::slice;
use crate::ffi::laux::{luaL_checkinteger, luaL_checklstring, luaL_checknumber};
use crate::util::{lua_rust_error, SimpleDrop};
use crate::vm::Stack;

/// This trait represents a function parameter.
pub trait FromParam: Sized + SimpleDrop {
    fn from_lua(stack: &Stack) -> Self;
}

impl FromParam for &str {
    fn from_lua(stack: &Stack) -> Self {
        unsafe {
            let mut len: usize = 0;
            let str = luaL_checklstring(stack.as_ptr(), stack.pop(), &mut len as _);
            let slice = slice::from_raw_parts(str as *const u8, len);
            match std::str::from_utf8(slice){
                Ok(v) => v,
                Err(e) => {
                    lua_rust_error(stack.as_ptr(), e);
                }
            }
        }
    }
}

macro_rules! impl_integer {
    ($($t: ty),*) => {
        $(
            impl FromParam for $t {
                fn from_lua(stack: &Stack) -> Self {
                    unsafe {
                        luaL_checkinteger(stack.as_ptr(), stack.pop()) as _
                    }
                }
            }
        )*
    };
}

#[cfg(target_pointer_width = "64")]
impl FromParam for i64 {
    fn from_lua(stack: &Stack) -> Self {
        unsafe {
            luaL_checkinteger(stack.as_ptr(), stack.pop()) as _
        }
    }
}

#[cfg(target_pointer_width = "64")]
impl FromParam for u64 {
    fn from_lua(stack: &Stack) -> Self {
        unsafe {
            luaL_checkinteger(stack.as_ptr(), stack.pop()) as _
        }
    }
}

impl_integer!(i8, u8, i16, u16, i32, u32);

impl FromParam for f64 {
    fn from_lua(stack: &Stack) -> Self {
        unsafe {
            luaL_checknumber(stack.as_ptr(), stack.pop()) as _
        }
    }
}

impl FromParam for f32 {
    fn from_lua(stack: &Stack) -> Self {
        unsafe {
            luaL_checknumber(stack.as_ptr(), stack.pop()) as _
        }
    }
}

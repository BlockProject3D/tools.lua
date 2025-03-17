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

use std::error::Error;
use std::ffi::{CStr, CString};
use crate::ffi::laux::luaL_loadstring;
use crate::ffi::lua::{lua_error, lua_pushlstring, State, ThreadStatus};

#[derive(Debug, PartialEq, Eq)]
pub enum TypeName {
    Some(&'static str),
    None
}

pub trait LuaType {
    /// Returns the closest rust type matching this lua value.
    fn lua_type() -> Vec<TypeName> {
        vec![TypeName::Some(std::any::type_name::<Self>())]
    }
}

impl<T: LuaType> LuaType for Option<T> {
    fn lua_type() -> Vec<TypeName> {
        let mut v = T::lua_type();
        v.push(TypeName::None);
        v
    }
}

pub unsafe fn lua_rust_error<E: Error>(l: State, error: E) -> ! {
    // At this point the function is assumed to be a non-POF (error and String).
    let s = format!("rust error: {}", error);
    lua_pushlstring(l, s.as_ptr() as _, s.len());
    // Drop both the error and the error string.
    // Very important as lua_error does not return.
    drop(error);
    drop(s);
    // Now the function should be back what Rust calls a POF.
    lua_error(l);
    // If this is reached, then lua_error has silently failed.
    std::unreachable!()
}

pub trait LoadCode {
    fn load_code(&self, l: State) -> ThreadStatus;
}

impl LoadCode for &CStr {
    #[inline(always)]
    fn load_code(&self, l: State) -> ThreadStatus {
        unsafe { luaL_loadstring(l, self.as_ptr()) }
    }
}

impl LoadCode for &str {
    fn load_code(&self, l: State) -> ThreadStatus {
        let s = CString::new(*self);
        match s {
            Ok(v) => {
                (&*v).load_code(l)
            }
            Err(_) => ThreadStatus::ErrSyntax
        }
    }
}

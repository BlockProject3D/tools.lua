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

use crate::vm::userdata::NameConvert;
use bp3d_util::string::BufTools;
use itertools::Itertools;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

fn to_string_lossy(bytes: Cow<[u8]>) -> Cow<str> {
    match bytes {
        Cow::Borrowed(v) => String::from_utf8_lossy(v),
        Cow::Owned(v) => String::from(&*String::from_utf8_lossy(&*v)).into(),
    }
}

pub struct Snake;

impl NameConvert for Snake {
    fn name_convert(&self, name: &'static CStr) -> Cow<'static, CStr> {
        Cow::Borrowed(name)
    }
}

pub struct Camel;

impl NameConvert for Camel {
    fn name_convert(&self, name: &'static CStr) -> Cow<'static, CStr> {
        let s = match name.to_str() {
            Ok(v) => v,
            // Return the same unconverted string if we failed.
            Err(_) => return Cow::Borrowed(name),
        };
        let s: String = s
            .split("_")
            .enumerate()
            .map(|(i, v)| {
                if i != 0 {
                    v.as_bytes().capitalise_ascii()
                } else {
                    v.as_bytes().into()
                }
            })
            .map(to_string_lossy)
            .join("")
            .into();
        CString::new(s).unwrap().into()
    }
}

pub struct Pascal;

impl NameConvert for Pascal {
    fn name_convert(&self, name: &'static CStr) -> Cow<'static, CStr> {
        let s = match name.to_str() {
            Ok(v) => v,
            // Return the same unconverted string if we failed.
            Err(_) => return Cow::Borrowed(name),
        };
        let s: String = s
            .split("_")
            .map(|v| v.as_bytes().capitalise_ascii())
            .map(to_string_lossy)
            .join("")
            .into();
        CString::new(s).unwrap().into()
    }
}

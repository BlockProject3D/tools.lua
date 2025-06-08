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

use crate::ffi::lua::Type;
use std::ffi::c_char;

pub const STRING_BUF_LEN: usize = 4096;

#[derive(Copy, Clone)]
#[repr(i32)]
pub enum ErrorType {
    None = 0,
    Utf8 = -1,
    Type = -2,
    Syntax = -3,
    Runtime = -4,
    Memory = -5,
    Unknown = -6,
    Error = -7,
    Null = -8,
    MultiValue = -9,
    UnsupportedType = -10,
    Loader = -11,
    ParseFloat = -12,
    ParseInt = -13,
    UserDataArgsEmpty = 1,
    UserDataMutViolation = 2,
    UserDataGc = 3,
    UserDataIndex = 4,
    UserDataMetatable = 5,
    UserDataMultiValueField = 6,
    UserDataAlreadyRegistered = 7,
    UserDataAlignment = 8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Utf8Error {
    pub ty: ErrorType,
    pub valid_up_to: usize,
    pub error_len: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TypeError {
    pub ty: ErrorType,
    pub expected: Type,
    pub actual: Type,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UnsupportedType {
    pub ty: ErrorType,
    pub actual: Type,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct String {
    pub ty: ErrorType,
    pub data: [u8; STRING_BUF_LEN],
    pub len: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StaticString {
    pub ty: ErrorType,
    pub data: *const c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Alignment {
    pub ty: ErrorType,
    pub alignment: usize,
}

#[repr(C)]
pub union Error {
    pub ty: ErrorType,
    pub string: String,
    pub type_mismatch: TypeError,
    pub utf8: Utf8Error,
    pub unsupported_type: UnsupportedType,
    pub static_string: StaticString,
    pub alignment: Alignment,
}

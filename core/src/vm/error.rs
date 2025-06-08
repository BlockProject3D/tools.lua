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
use bp3d_util::simple_error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct TypeError {
    pub expected: Type,
    pub actual: Type,
}

impl Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected {:?}, got {:?}", self.expected, self.actual)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    traceback: String,
    index: usize,
}

impl RuntimeError {
    pub fn new(traceback: String) -> Self {
        let id = traceback.find('\n').unwrap();
        Self {
            traceback,
            index: id,
        }
    }

    pub fn msg(&self) -> &str {
        &self.traceback[..self.index]
    }

    pub fn stacktrace(&self) -> &str {
        &self.traceback[self.index + 1..]
    }

    pub fn backtrace(&self) -> &str {
        &self.traceback
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Utf8Error {
    // A re-usable error type is needed for modules so duplicate the one from std.
    pub valid_up_to: usize,
    pub error_len: Option<u8>,
}

impl From<std::str::Utf8Error> for Utf8Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Utf8Error {
            valid_up_to: value.valid_up_to(),
            error_len: value.error_len().map(|v| v as u8),
        }
    }
}

impl Utf8Error {
    pub const fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    pub const fn error_len(&self) -> Option<usize> {
        match self.error_len {
            Some(len) => Some(len as usize),
            None => None,
        }
    }
}

impl Display for Utf8Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(error_len) = self.error_len {
            write!(
                f,
                "invalid utf-8 sequence of {} bytes from index {}",
                error_len, self.valid_up_to
            )
        } else {
            write!(
                f,
                "incomplete utf-8 byte sequence from index {}",
                self.valid_up_to
            )
        }
    }
}

simple_error! {
    pub Error {
        InvalidUtf8(Utf8Error) => "invalid UTF8 string: {}",
        Type(TypeError) => "type error: {}",
        Syntax(String) => "syntax error: {}",
        Runtime(RuntimeError) => "runtime error: {}",
        UncatchableRuntime(RuntimeError) => "uncatchable runtime error: {}",
        Memory => "memory allocation error",
        Unknown => "unknown error",
        Error => "error in error handler",
        Null => "string contains a null character",
        MultiValue => "only one value is supported by this API",
        UserData(crate::vm::userdata::Error) => "userdata: {}",
        UnsupportedType(Type) => "unsupported lua type: {:?}",
        Loader(String) => "loader error: {}",
        ParseFloat => "parse float error",
        ParseInt => "parse int error"
    }
}

impl Error {
    pub fn into_runtime(self) -> Option<RuntimeError> {
        match self {
            Error::Runtime(e) => Some(e),
            _ => None,
        }
    }
}

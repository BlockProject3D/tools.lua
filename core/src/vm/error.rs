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
use std::str::Utf8Error;

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

simple_error! {
    pub Error {
        InvalidUtf8(Utf8Error) => "invalid UTF8 string: {}",
        Type(TypeError) => "type error: {}",
        Syntax(String) => "syntax error: {}",
        Runtime(RuntimeError) => "runtime error: {}",
        Memory => "memory allocation error",
        Unknown => "unknown error",
        Error => "error in error handler",
        Null => "string contains a null character",
        MultiValue => "only one value is supported by this API",
        UserData(crate::vm::userdata::Error) => "userdata: {}",
        UnsupportedType(Type) => "unsupported lua type: {:?}",
        Loader(String) => "loader error: {}"
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

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

pub mod error;

/// The BP3D Lua & LuaJIT version.
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the time library used by bp3d-lua.
pub static TIME_VERSION: &str = "0.3.41";

/// The macro which generates a plugin entry point.
pub use bp3d_lua_codegen::decl_lua_plugin;

/// Helper function to run the [register](crate::libs::Lib::register) function of a
/// [Lib](crate::libs::Lib).
///
/// This function automatically translates the Rust result type to the C FFI compatible type.
pub fn run_lua_register(vm: &crate::vm::Vm, lib: impl crate::libs::Lib, error: &mut error::Error) -> bool {
    use crate::vm::error::Error;
    use std::fmt::Write;
    use bp3d_util::format::MemBufStr;
    let res = lib.register(vm);
    match res {
        Ok(()) => true,
        Err(e) => {
            match e {
                Error::InvalidUtf8(e) => {
                    error.ty = error::ErrorType::Utf8;
                    // Option is not FFI safe, so use i16.
                    error.utf8.error_len = e.error_len().map(|v| v as i16).unwrap_or(-1);
                    error.utf8.valid_up_to = e.valid_up_to();
                }
                Error::Type(e) => {
                    error.ty = error::ErrorType::Type;
                    error.type_mismatch.actual = e.actual;
                    error.type_mismatch.expected = e.expected;
                }
                Error::Syntax(e) => {
                    error.ty = error::ErrorType::Syntax;
                    let mut msg = unsafe { MemBufStr::wrap(&mut error.string.len, &mut error.string.data) };
                    let _ = write!(msg, "{}", e);
                }
                Error::Runtime(e) => {
                    error.ty = error::ErrorType::Runtime;
                    let mut msg = unsafe { MemBufStr::wrap(&mut error.string.len, &mut error.string.data) };
                    let _ = write!(msg, "{}", e);
                }
                Error::Memory => error.ty = error::ErrorType::Memory,
                Error::Unknown => error.ty = error::ErrorType::Unknown,
                Error::Error => error.ty = error::ErrorType::Error,
                Error::Null => error.ty = error::ErrorType::Null,
                Error::MultiValue => error.ty = error::ErrorType::MultiValue,
                Error::UserData(e) => match e {
                    crate::vm::userdata::Error::ArgsEmpty => error.ty = error::ErrorType::UserDataArgsEmpty,
                    crate::vm::userdata::Error::MutViolation(e) => {
                        error.ty = error::ErrorType::UserDataMutViolation;
                        error.static_string.data = e.as_ptr();
                    }
                    crate::vm::userdata::Error::Gc => error.ty = error::ErrorType::UserDataGc,
                    crate::vm::userdata::Error::Index => error.ty = error::ErrorType::UserDataIndex,
                    crate::vm::userdata::Error::Metatable => error.ty = error::ErrorType::UserDataMetatable,
                    crate::vm::userdata::Error::MultiValueField => error.ty = error::ErrorType::UserDataMultiValueField,
                    crate::vm::userdata::Error::AlreadyRegistered(e) => {
                        error.ty = error::ErrorType::UserDataAlreadyRegistered;
                        error.static_string.data = e.as_ptr();
                    }
                    crate::vm::userdata::Error::Alignment(e) => {
                        error.ty = error::ErrorType::UserDataAlignment;
                        error.alignment.alignment = e;
                    }
                }
                Error::UnsupportedType(e) => {
                    error.ty = error::ErrorType::UnsupportedType;
                    error.unsupported_type.actual = e;
                }
                Error::Loader(e) => {
                    error.ty = error::ErrorType::Loader;
                    let mut msg = unsafe { MemBufStr::wrap(&mut error.string.len, &mut error.string.data) };
                    let _ = write!(msg, "{}", e);
                }
                Error::ParseInt => error.ty = error::ErrorType::ParseInt,
                Error::ParseFloat => error.ty = error::ErrorType::ParseFloat,
                Error::UncatchableRuntime(e) => {
                    error.ty = error::ErrorType::UncatchableRuntime;
                    let mut msg = unsafe { MemBufStr::wrap(&mut error.string.len, &mut error.string.data) };
                    let _ = write!(msg, "{}", e);
                }
            }
            false
        }
    }
}

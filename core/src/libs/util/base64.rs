// Copyright (c) 2026, BlockProject 3D
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

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::BASE64_STANDARD;
use crate::decl_lib_func;
use crate::libs::Lib;
use crate::util::Namespace;
use crate::vm::function::types::RFunction;

decl_lib_func! {
    fn decode(src: &str) -> Result<Box<[u8]>, base64::DecodeError> {
        BASE64_STANDARD.decode(src).map(|v| v.into_boxed_slice())
    }
}

decl_lib_func! {
    fn encode(src: &[u8]) -> String {
        BASE64_STANDARD.encode(src)
    }
}

decl_lib_func! {
    fn decode_url_safe(src: &str) -> Result<Box<[u8]>, base64::DecodeError> {
        URL_SAFE.decode(src).map(|v| v.into_boxed_slice())
    }
}

decl_lib_func! {
    fn encode_url_safe(src: &[u8]) -> String {
        URL_SAFE.encode(src)
    }
}

pub struct Base64;

impl Lib for Base64 {
    const NAMESPACE: &'static str = "bp3d.util.base64";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("encode", RFunction::wrap(encode)),
            ("decode", RFunction::wrap(decode)),
            ("encodeUrlSafe", RFunction::wrap(encode_url_safe)),
            ("decodeUrlSafe", RFunction::wrap(decode_url_safe))
        ])
    }
}

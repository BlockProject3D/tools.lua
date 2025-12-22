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

use crate::decl_lib_func;
use crate::libs::Lib;
use crate::util::Namespace;
use crate::vm::function::types::RFunction;
use crate::vm::table::Table;
use bp3d_util::string::BufTools;
use std::borrow::Cow;

decl_lib_func! {
    fn starts_with(src: &[u8], prefix: &[u8]) -> bool {
        src.starts_with(prefix)
    }
}

decl_lib_func! {
    fn ends_with(src: &[u8], suffix: &[u8]) -> bool {
        src.ends_with(suffix)
    }
}

decl_lib_func! {
    fn contains(src: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() {
            return true;
        }
        src.windows(needle.len()).any(|window| window == needle)
    }
}

decl_lib_func! {
    fn split<'a>(vm: &Vm, src: &[u8], pattern: u8) -> crate::vm::Result<Table<'a>> {
        let split = src.split(|v| *v == pattern);
        let mut tbl = Table::new(vm);
        for (i, v) in split.enumerate() {
            // Indices starts at 1 in lua.
            tbl.set(i + 1, v)?;
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    fn capitalise(src: &[u8]) -> Cow<'_, [u8]> {
        src.capitalise_ascii()
    }
}

decl_lib_func! {
    fn decapitalise(src: &[u8]) -> Cow<'_, [u8]> {
        src.decapitalise_ascii()
    }
}

pub struct String;

impl Lib for String {
    const NAMESPACE: &'static str = "bp3d.util.string";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("contains", RFunction::wrap(contains)),
            ("split", RFunction::wrap(split)),
            ("capitalise", RFunction::wrap(capitalise)),
            ("decapitalise", RFunction::wrap(decapitalise)),
            ("startsWith", RFunction::wrap(starts_with)),
            ("endsWith", RFunction::wrap(ends_with)),
        ])
    }
}

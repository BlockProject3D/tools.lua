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
use crate::ffi::lua::RawNumber;
use crate::libs::Lib;
use crate::util::Namespace;
use crate::vm::function::types::RFunction;
use crate::vm::value::any::Any;
use crate::vm::value::types::{Int53, UInt53};

decl_lib_func! {
    fn eq(a: RawNumber, b: RawNumber, epsilon: RawNumber) -> bool {
        (a - b).abs() <= epsilon
    }
}

decl_lib_func! {
    fn parsenumber(value: &str) -> (Option<RawNumber>, Option<String>) {
        match value.parse() {
            Ok(n) => (Some(n), None),
            Err(e) => (None, Some(e.to_string()))
        }
    }
}

decl_lib_func! {
    fn parseint64(value: &str) -> (Option<i64>, Option<String>) {
        match value.parse() {
            Ok(n) => (Some(n), None),
            Err(e) => (None, Some(e.to_string()))
        }
    }
}

decl_lib_func! {
    fn parseuint64(value: &str) -> (Option<u64>, Option<String>) {
        match value.parse() {
            Ok(n) => (Some(n), None),
            Err(e) => (None, Some(e.to_string()))
        }
    }
}

decl_lib_func! {
    fn toistring(val: Any) -> crate::vm::Result<String> {
        let val = val.to_integer()?;
        Ok(val.to_string())
    }
}

decl_lib_func! {
    fn toustring(val: Any) -> crate::vm::Result<String> {
        let val = val.to_uinteger()?;
        Ok(val.to_string())
    }
}

pub struct Num;

impl Lib for Num {
    const NAMESPACE: &'static str = "bp3d.util.num";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([("UINT53_MAX", UInt53::MAX), ("UINT53_MIN", UInt53::MIN)])?;
        namespace.add([("INT53_MAX", Int53::MAX), ("INT53_MIN", Int53::MIN)])?;
        namespace.add([("UINT64_MAX", u64::MAX), ("UINT64_MIN", u64::MIN)])?;
        namespace.add([("INT64_MAX", i64::MAX), ("INT64_MIN", i64::MIN)])?;
        namespace.add([("NAN", f64::NAN), ("EPSILON", f64::EPSILON)])?;
        namespace.add([
            ("toistring", RFunction::wrap(toistring)),
            ("toustring", RFunction::wrap(toustring)),
            ("eq", RFunction::wrap(eq)),
            ("parsenumber", RFunction::wrap(parsenumber)),
            ("parseint64", RFunction::wrap(parseint64)),
            ("parseuint64", RFunction::wrap(parseuint64)),
        ])
    }
}

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

use crate::libs::interface::Lib;
use crate::vm::core::load::{Code, Script};
use crate::vm::function::types::RFunction;
use crate::util::Namespace;
use crate::vm::value::any::{AnyParam, UncheckedAnyReturn};
use crate::vm::value::Function;
use crate::{decl_closure, decl_lib_func};
use bp3d_util::simple_error;
use std::path::{Path, PathBuf};

decl_lib_func! {
    fn run_string(vm: &Vm, s: &str, chunkname: Option<&str>) -> crate::vm::Result<UncheckedAnyReturn> {
        let top = vm.top();
        let ret = match chunkname {
            None => vm.run_code::<AnyParam>(s),
            Some(name) => vm.run::<AnyParam>(Code::new(name, s.as_bytes()))
        };
        ret.map(|_| unsafe { UncheckedAnyReturn::new(vm, (vm.top() - top) as _) })
    }
}

decl_lib_func! {
    fn load_string<'a>(vm: &Vm, s: &str, chunkname: Option<&str>) -> (Option<Function<'a>>, Option<String>) {
        match chunkname {
            None => match vm.load_code(s) {
                Ok(v) => (Some(v), None),
                Err(v) => (None, Some(v.to_string()))
            },
            Some(name) => match vm.load(Code::new(name, s.as_bytes())) {
                Ok(v) => (Some(v), None),
                Err(v) => (None, Some(v.to_string()))
            }
        }
    }
}

simple_error! {
    Error {
        EscapeChroot(String) => "invalid path {}: attempt to escape chroot",
        (impl From) Io(std::io::Error) => "io error: {}",
        (impl From) Vm(crate::vm::error::Error) => "lua error: {}"
    }
}

fn parse_lua_path(chroot: &Path, path: &str) -> Result<PathBuf, Error> {
    let iter = path.split("/");
    let mut cur_path = Vec::new();
    for component in iter {
        if component == ".." {
            if cur_path.pop().is_none() {
                return Err(Error::EscapeChroot(path.into()));
            }
        } else if component != "." {
            cur_path.push(component);
        }
    }
    Ok(chroot.join(path))
}

decl_closure! {
    fn load_file<'a> |chroot: &Path| (vm: &Vm, path: &str) -> (Option<Function<'a>>, Option<String>) {
        let path = match parse_lua_path(chroot, path) {
            Ok(v) => v,
            Err(e) => return (None, Some(e.to_string()))
        };
        let script = match Script::from_path(path) {
            Ok(v) => v,
            Err(e) => return (None, Some(e.to_string()))
        };
        match vm.load(script) {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e.to_string()))
        }
    }
}

decl_closure! {
    fn run_file<'a> |chroot: &Path| (vm: &Vm, path: &str) -> Result<UncheckedAnyReturn, Error> {
        let path = parse_lua_path(chroot, path)?;
        let script = Script::from_path(path)?;
        let top = vm.top();
        vm.run::<AnyParam>(script)?;
        unsafe { Ok(UncheckedAnyReturn::new(vm, (vm.top() - top) as _)) }
    }
}

pub struct Load<'a>(pub Option<&'a Path>);

impl Lib for Load<'_> {
    const NAMESPACE: &'static str = "bp3d.lua";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("runString", RFunction::wrap(run_string)),
            ("loadString", RFunction::wrap(load_string)),
        ])?;
        if let Some(chroot) = self.0 {
            namespace.add([
                ("loadFile", load_file(chroot)),
                ("runFile", run_file(chroot)),
            ])?;
        }
        Ok(())
    }
}

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

use crate::decl_closure;
use crate::libs::interface::Lib;
use crate::util::Namespace;
use crate::vm::closure::arc::{Arc, Shared};
use crate::vm::core::debug::DebugRegistry;
use crate::vm::value::any::{AnyParam, UncheckedAnyReturn};
use crate::vm::Vm;
use bp3d_util::simple_error;
use std::collections::HashMap;
use std::sync::RwLock;

simple_error! {
    pub Error {
        (impl From) Vm(crate::vm::error::Error) => "lua error: {}",
        InvalidSyntax => "invalid syntax for require",
        UnknownSource(String) => "unknown source name {}"
    }
}

pub trait Source: Send + Sync {
    fn run(&self, vm: &Vm, path: &str, full_path: &str) -> crate::vm::Result<AnyParam>;
}

#[derive(Default)]
pub struct Provider(RwLock<HashMap<String, Box<dyn Source>>>);

impl Provider {
    pub fn new() -> Self {
        Provider::default()
    }

    pub fn add_source<S: Source + 'static>(&self, name: String, source: S) {
        let mut guard = self.0.write().unwrap();
        guard.insert(name, Box::new(source));
    }

    pub fn require(&self, vm: &Vm, path: &str) -> Result<AnyParam, Error> {
        let id = path.find('.').ok_or(Error::InvalidSyntax)?;
        let source = &path[..id];
        let guard = self.0.read().unwrap();
        let src = guard
            .get(source)
            .ok_or_else(|| Error::UnknownSource(source.into()))?;
        let ret = src.run(vm, &path[id + 1..], path)?;
        Ok(ret)
    }
}

decl_closure! {
    fn require |provider: Arc<Provider>| (vm: &Vm, path: &str) -> Result<UncheckedAnyReturn, Error> {
        let top = vm.top();
        provider.require(vm, path)?;
        unsafe { Ok(UncheckedAnyReturn::new(vm, (vm.top() - top) as _)) }
    }
}

pub struct Require(pub Shared<Provider>);

impl Lib for Require {
    const NAMESPACE: &'static str = "bp3d.lua";

    fn load(&self, _: &mut Namespace) -> crate::vm::Result<()> {
        std::unreachable!()
    }

    fn register(&self, vm: &Vm) -> crate::vm::Result<()> {
        DebugRegistry::add::<Require, _>(vm);
        let rc = Arc::from_rust(vm, self.0.clone());
        let mut namespace = Namespace::new(vm, "bp3d.lua")?;
        namespace.add([("require", require(rc))])
    }
}

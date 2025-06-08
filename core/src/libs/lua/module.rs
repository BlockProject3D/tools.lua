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

use crate::decl_userdata_mut;
use crate::libs::Lib;
use crate::util::module::ModuleManager;
use crate::util::module::Result;
use crate::util::Namespace;
use crate::vm::Vm;
use bp3d_os::module::library::types::VirtualLibrary;
use std::path::PathBuf;

pub struct Module {
    builtins: &'static [&'static VirtualLibrary],
    search_paths: Vec<PathBuf>,
}

impl Module {
    pub fn new(builtins: &'static [&'static VirtualLibrary]) -> Self {
        Self {
            builtins,
            search_paths: Vec::new(),
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) -> &mut Self {
        self.search_paths.push(path);
        self
    }
}

struct ModuleManagerWrapper(ModuleManager);

decl_userdata_mut! {
    impl ModuleManagerWrapper {
        fn load(this: &mut ModuleManagerWrapper, vm: &Vm, lib: &str, plugin: &str) -> Result<()> {
            this.0.load(lib, plugin, vm)
        }
    }
}

impl Lib for Module {
    const NAMESPACE: &'static str = "";

    fn load(&self, _: &mut Namespace) -> crate::vm::Result<()> {
        unreachable!()
    }

    fn register(&self, vm: &Vm) -> crate::vm::Result<()> {
        vm.register_userdata::<ModuleManagerWrapper>(crate::vm::userdata::case::Camel)?;
        let mut manager = ModuleManager::new(self.builtins);
        for search_path in &self.search_paths {
            manager.add_search_path(search_path.clone());
        }
        vm.set_global(c"MODULES", ModuleManagerWrapper(manager))
    }
}

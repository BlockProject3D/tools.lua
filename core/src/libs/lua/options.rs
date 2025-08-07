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

use crate::libs::lua::base::Base;
use crate::libs::lua::call::Call;
use crate::libs::lua::load::Load;
use crate::libs::lua::require::{Provider, Require};
use crate::libs::Lib;
use std::path::Path;
use crate::vm::closure::arc::Shared;

#[derive(Default)]
pub struct Lua<'a> {
    pub(super) load_chroot_path: Option<&'a Path>,
    pub(super) provider: Option<Shared<Provider>>,
}

impl<'a> Lua<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_chroot_path(mut self, path: &'a Path) -> Self {
        self.load_chroot_path = Some(path);
        self
    }

    pub fn provider(mut self, provider: Shared<Provider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn build(self) -> impl Lib + 'a {
        (
            Base,
            Call,
            Load(self.load_chroot_path),
            Require(self.provider.unwrap_or_default()),
        )
    }
}

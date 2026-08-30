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

use crate::Target;
use crate::build::{Build, Lib, Linux, MacOS, Windows};
use std::io::Error;
use std::path::{Path, PathBuf};

pub struct BuildInfoBase<'a> {
    pub dynamic: bool,
    pub target_name: &'a str,
    pub build_dir: &'a Path,
    pub manifest: &'a Path,
}

pub struct BuildInfo<'a> {
    base: BuildInfoBase<'a>,
    target_dir: PathBuf,
    crate_version: String,
}

const VERSION: &str = "version = \"";

impl<'a> BuildInfo<'a> {
    pub fn new(base: BuildInfoBase<'a>) -> std::io::Result<Self> {
        let manifest = std::fs::read_to_string(base.manifest)?;
        let target_dir = base.build_dir.join("../../../..");
        let start = manifest
            .find(VERSION)
            .ok_or(Error::other("failed to find crate version"))?;
        let version = &manifest[start + VERSION.len()..];
        let end = version
            .find("\"")
            .ok_or(Error::other("failed to find crate version"))?;
        Ok(Self {
            base,
            target_dir,
            crate_version: String::from(&version[..end]),
        })
    }

    pub fn build_dir(&self) -> &Path {
        self.base.build_dir
    }

    pub fn dynamic(&self) -> bool {
        self.base.dynamic
    }

    pub fn target(&self) -> Target {
        Target::get(self.base.target_name)
    }

    pub fn target_name(&self) -> &str {
        self.base.target_name
    }

    pub fn target_dir(&self) -> &Path {
        &self.target_dir
    }

    pub fn version(&self) -> &str {
        &self.crate_version
    }

    pub fn build(self) -> std::io::Result<Lib> {
        match self.target() {
            Target::MacAmd64 | Target::MacAarch64 => MacOS::run(&self),
            Target::Linux => Linux::run(&self),
            Target::Windows => Windows::run(&self),
            Target::Unsupported => Err(Error::other("unsupported target")),
        }
    }
}

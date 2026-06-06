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

use crate::util::CommandRunner;
use bp3d_os::fs::CopyOptions;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Summary {
    patches: Vec<String>,
}

impl Summary {
    pub fn write(&self, out: &Path) -> std::io::Result<()> {
        let mut file = File::create(out)?;
        writeln!(&mut file, "const PATCH_LIST: &[&str] = &[")?;
        for patch in &self.patches {
            writeln!(&mut file, "    \"{}\",", patch)?;
        }
        writeln!(&mut file, "];")?;
        Ok(())
    }
}

pub struct Patch {
    src_path: PathBuf,
    patch_dir: PathBuf,
    patch_list: Vec<String>,
}

impl Patch {
    pub fn new(patch_dir: &Path, luajit_src: &Path) -> std::io::Result<Patch> {
        let patch_dir = bp3d_os::fs::get_absolute_path(patch_dir)?;
        let src_path = bp3d_os::fs::get_absolute_path(luajit_src)?;
        CommandRunner::new("failed to revert").run(
            Command::new("git")
                .args(["checkout", "."])
                .current_dir(&src_path),
        )?;
        Ok(Patch {
            patch_dir,
            src_path,
            patch_list: Vec::new(),
        })
    }

    pub fn apply(&mut self, name: &str) -> std::io::Result<()> {
        CommandRunner::new("failed to patch").run(
            Command::new("git")
                .args([
                    OsStr::new("apply"),
                    self.patch_dir.join(format!("{}.patch", name)).as_os_str(),
                ])
                .current_dir(&self.src_path),
        )?;
        self.patch_list.push(name.into());
        Ok(())
    }

    pub fn get_patch_list(&self) -> impl Iterator<Item = &str> {
        self.patch_list.iter().map(|v| &**v)
    }

    pub fn apply_all<'a>(
        mut self,
        patches: impl IntoIterator<Item = &'a str>,
        out_path: &Path,
    ) -> std::io::Result<Summary> {
        for patch in patches {
            self.apply(patch)?;
        }
        if !out_path.is_dir() {
            bp3d_os::fs::copy(
                &self.src_path,
                out_path,
                CopyOptions::new().exclude(OsStr::new(".git")),
            )?;
        }
        Ok(Summary {
            patches: self.get_patch_list().map(String::from).collect(),
        })
    }
}

impl Drop for Patch {
    fn drop(&mut self) {
        CommandRunner::new("failed to revert")
            .run(
                Command::new("git")
                    .args(["checkout", "."])
                    .current_dir(&self.src_path),
            )
            .unwrap();
    }
}

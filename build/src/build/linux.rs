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

use crate::BuildInfo;
use crate::build::Build;
use crate::build::interface::Lib;
use crate::util::CommandRunner;
use std::process::Command;

pub struct Linux;

impl Build for Linux {
    fn build(info: &BuildInfo, runner: &CommandRunner) -> std::io::Result<()> {
        let soname = format!("TARGET_SONAME=libbp3d-luajit-{}.so", info.version());
        runner.run(
            Command::new("make")
                .arg(soname)
                .current_dir(info.build_dir()),
        )
    }

    fn post_build(info: &BuildInfo, _: &CommandRunner) -> std::io::Result<()> {
        let filename = format!("libbp3d-luajit-{}.so", info.version());
        let path_to_so = info.build_dir().join("src").join("libluajit.so");
        let path_to_dylib = info.build_dir().join(&filename);
        std::fs::copy(&path_to_so, path_to_dylib)?;
        let path_to_dylib2 = info.target_dir().join(filename);
        std::fs::copy(&path_to_so, path_to_dylib2)?;
        std::fs::remove_file(path_to_so.join("libluajit.so"))?;
        Ok(())
    }

    fn get_linked_lib(info: &BuildInfo) -> Lib {
        if info.dynamic() {
            let name = format!("bp3d-luajit-{}", info.version());
            Lib {
                name,
                path: info.build_dir().into(),
                dynamic: true,
            }
        } else {
            Lib {
                name: "luajit".into(),
                path: info.build_dir().join("src"),
                dynamic: false,
            }
        }
    }
}

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
use std::io::{Error, ErrorKind};
use std::process::Command;

pub struct Windows;

impl Build for Windows {
    fn build(info: &BuildInfo, runner: &CommandRunner) -> std::io::Result<()> {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("msvcbuild.bat");
        let cl = cc::windows_registry::find_tool(info.target_name(), "cl.exe")
            .ok_or(Error::other("unable to find cl.exe"))?;
        for (k, v) in cl.env() {
            cmd.env(k, v);
        }
        if info.target_name().contains("aarch64") {
            cmd.env("VSCMD_ARG_HOST_ARCH", "arm64");
            cmd.env("VSCMD_ARG_TGT_ARCH", "arm64");
        }
        if !info.dynamic() {
            cmd.arg("static");
        }
        let dllname = format!("libbp3d-luajit-{}.dll", info.version());
        let libname = format!("libbp3d-luajit-{}.lib", info.version());
        cmd.env("LJDLLNAME", dllname);
        cmd.env("LJLIBNAME", libname);
        runner.run(cmd.current_dir(info.build_dir().join("src")))
    }

    fn post_build(info: &BuildInfo, _: &CommandRunner) -> std::io::Result<()> {
        if !info.dynamic() {
            //Nothing to be done in non-dynamic builds.
            return Ok(());
        }
        let dllname = format!("libbp3d-luajit-{}.dll", info.version());
        let path_to_dll = info.build_dir().join("src").join(&dllname);
        let path_to_dylib = info.build_dir().join(&dllname);
        std::fs::copy(&path_to_dll, path_to_dylib)?;
        let path_to_dylib2 = info.target_dir().join(dllname);
        std::fs::copy(path_to_dll, path_to_dylib2)?;
        Ok(())
    }

    fn get_linked_lib(info: &BuildInfo) -> Lib {
        let libname = format!("libbp3d-luajit-{}", info.version());
        Lib {
            name: libname,
            path: info.build_dir().join("src"),
            dynamic: info.dynamic(),
        }
    }
}

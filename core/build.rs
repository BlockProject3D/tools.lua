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

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus};
use bp3d_os::fs::CopyOptions;
use phf::phf_map;

pub enum Target {
    MacAmd64,
    MacAarch64,
    Linux,
    Windows,
    Unsupported
}

static TARGET_MAP: phf::Map<&'static str, Target> = phf_map! {
    "aarch64-apple-darwin" => Target::MacAarch64,
    "aarch64-unknown-linux-gnu" => Target::Linux,
    "i686-pc-windows-msvc" => Target::Windows,
    "x86_64-pc-windows-msvc" => Target::Windows,
    "x86_64-apple-darwin" => Target::MacAmd64,
    "x86_64-unknown-linux-gnu" => Target::Linux
};

fn run_command_in_luajit(text: &str, cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> ExitStatus {
    let path = bp3d_os::fs::get_absolute_path("../").expect("Failed to acquire current path");
    std::process::Command::new(cmd)
        .args(args)
        .current_dir(path.join("LuaJIT")).status().expect(text)
}

fn build_luajit(build_dir: &Path) {
    let target_name = std::env::var("TARGET").expect("Failed to read build target");
    let target = TARGET_MAP.get(&target_name).unwrap_or(&Target::Unsupported);
    let cmd = match target {
        Target::MacAmd64 => Command::new("make")
            .env("MACOSX_DEPLOYMENT_TARGET", "10.11")
            .current_dir(build_dir).status(),
        Target::MacAarch64 => Command::new("make")
            .env("MACOSX_DEPLOYMENT_TARGET", "11.0")
            .current_dir(build_dir).status(),
        Target::Linux => Command::new("make")
            .current_dir(build_dir).status(),
        Target::Windows => {
            let mut cmd = Command::new("msvcbuild.bat");
            let cl = cc::windows_registry::find_tool(&target_name, "cl.exe").expect("failed to find cl");
            for (k, v) in cl.env() {
                cmd.env(k, v);
            }
            cmd.current_dir(build_dir.join("src")).status()
        },
        Target::Unsupported => panic!("Unsupported build target {}", target_name)
    }.expect("Failed to run build command");
    if !cmd.success() {
        panic!("Failed to build LuaJIT");
    }
}

fn main() {
    // Rerun this script if any of the patch files changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=patch");

    // Revert patch to LuaJIT source code and cleanup.
    run_command_in_luajit("Failed to revert LuaJIT patch", "git", &["checkout", "."]);

    // Create build directory.
    let out = std::env::var_os("OUT_DIR").expect("Failed to acquire output directory");
    let out_path = Path::new(&out).join("luajit-build");

    // Apply patches to LuaJIT source code.
    let path = bp3d_os::fs::get_absolute_path("../").expect("Failed to acquire current path");
    let result = run_command_in_luajit("Failed to patch LuaJIT", "git", &[OsStr::new("apply"), path.join("patch/lib_init.patch").as_os_str()]);
    if !result.success() {
        panic!("Failed to patch LuaJIT");
    }
    let result = run_command_in_luajit("Failed to patch LuaJIT", "git", &[OsStr::new("apply"), path.join("patch/lj_disable_jit.patch").as_os_str()]);
    if !result.success() {
        panic!("Failed to patch LuaJIT");
    }
    let result = run_command_in_luajit("Failed to patch LuaJIT", "git", &[OsStr::new("apply"), path.join("patch/disable_lua_load.patch").as_os_str()]);
    if !result.success() {
        panic!("Failed to patch LuaJIT");
    }

    // Copy the source directory to the build directory.
    println!("{}", out_path.display());
    if !out_path.is_dir() {
        bp3d_os::fs::copy(&path.join("LuaJIT"), &out_path, CopyOptions::new().exclude(OsStr::new(".git"))).expect("Failed to copy LuaJIT sources to build directory");
    }

    // Revert patch to LuaJIT source code and cleanup.
    run_command_in_luajit("Failed to revert LuaJIT patch", "git", &["checkout", "."]);

    // Build LuaJIT.
    build_luajit(&out_path);

    // Attempt to setup linkage.
    println!("cargo:rustc-link-search=native={}", out_path.join("src").display());
    println!("cargo:rustc-link-lib=static:-bundle,+whole-archive=luajit");
}

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
use std::process::ExitStatus;
use bp3d_os::fs::CopyOptions;

fn run_command_in_luajit(text: &str, cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> ExitStatus {
    let path = bp3d_os::fs::get_absolute_path("./").expect("Failed to acquire current path");
    std::process::Command::new(cmd)
        .args(args)
        .current_dir(path.join("LuaJIT")).status().expect(text)
}

fn main() {
    // Create build directory.
    let out = std::env::var_os("OUT_DIR").expect("Failed to acquire output directory");
    let out_path = Path::new(&out).join("luajit-build");
    std::fs::create_dir_all(&out_path).expect("Failed to create LuaJIT build directory");

    // Apply patch to LuaJIT source code.
    let path = bp3d_os::fs::get_absolute_path("./").expect("Failed to acquire current path");
    let result = run_command_in_luajit("Failed to patch LuaJIT", "git", &[OsStr::new("apply"), path.join("luajit.patch").as_os_str()]);
    if !result.success() {
        panic!("Failed to patch LuaJIT");
    }

    // Copy the source directory to the build directory.
    bp3d_os::fs::copy(&path.join("LuaJIT"), &out_path, CopyOptions::new()).expect("Failed to copy LuaJIT sources to build directory");

    // Revert patch to LuaJIT source code and cleanup.
    run_command_in_luajit("Failed to revert LuaJIT patch", "git", &["checkout", "."]);
}

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
use std::path::{Path, PathBuf};
use bp3d_os::fs::CopyOptions;
use bp3d_lua_build::{BuildInfo, Patch, Target};
use bp3d_lua_build::build::{Build, Lib, Linux, MacOS, Windows};

#[cfg(feature = "dynamic")]
const DYNAMIC: bool = true;
#[cfg(not(feature = "dynamic"))]
const DYNAMIC: bool = false;

fn apply_patches(out_path: &Path) -> std::io::Result<()> {
    let mut patch = Patch::new(&Path::new("..").join("patch"), &Path::new("..").join("LuaJIT"))?;
    patch.apply("lib_init")?; // Disable unsafe/un-sandboxed libs.
    patch.apply("lj_disable_jit")?; // Disable global JIT state changes from Lua code.
    patch.apply("disable_lua_load")?; // Disable loadstring, dostring, etc from base lib.
    patch.apply("lua_ext")?; // Ext library such as lua_ext_tab_len, etc.
    patch.apply("lua_load_no_bc")?; // Treat all inputs as strings (no bytecode allowed).

    // Copy the source directory to the build directory.
    println!("{}", out_path.display());
    if !out_path.is_dir() {
        bp3d_os::fs::copy(&Path::new("..").join("LuaJIT"), &out_path, CopyOptions::new().exclude(OsStr::new(".git")))?;
    }
    Ok(())
}

fn run_build(build_dir: &Path) -> std::io::Result<Lib> {
    let manifest = std::env::var_os("CARGO_MANIFEST_PATH").map(PathBuf::from).expect("Failed to read manifest path");
    let target_name = std::env::var("TARGET").expect("Failed to read build target");
    let info = BuildInfo::new(DYNAMIC, target_name, build_dir.into(), &manifest)?;
    match info.target() {
        Target::MacAmd64 | Target::MacAarch64 => MacOS::run(&info),
        Target::Linux => Linux::run(&info),
        Target::Windows => Windows::run(&info),
        Target::Unsupported => panic!("attempt to build on currently unsupported target")
    }
}

fn main() {
    // Rerun this script if any of the patch files changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=patch");

    // Create build directory.
    let out = std::env::var_os("OUT_DIR").expect("Failed to acquire output directory");
    let out_path = Path::new(&out).join("luajit-build");

    // Apply patches to LuaJIT source code.
    apply_patches(&out_path).expect("Failed to patch LuaJIT");

    // Copy the source directory to the build directory.
    println!("Internal LuaJIT build directory: {}", out_path.display());

    // Build LuaJIT.
    let lib = run_build(&out_path).expect("Failed to build LuaJIT");

    // Attempt to setup linkage.
    println!("cargo:rustc-link-search=native={}", lib.path.display());
    if lib.dynamic {
        println!("cargo:rustc-link-lib=dylib={}", lib.name);
    } else {
        println!("cargo:rustc-link-lib=static={}", lib.name);
    }
}

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

use bp3d_lua_build::build::Lib;
use bp3d_lua_build::{BuildInfo, BuildInfoBase, Patch};
use std::path::{Path, PathBuf};

#[cfg(feature = "dynamic")]
const DYNAMIC: bool = true;
#[cfg(not(feature = "dynamic"))]
const DYNAMIC: bool = false;

const PATCH_LIST: &[&str] = &[
    "lib_init",              // Disable unsafe/un-sandboxed libs.
    "lj_disable_jit",        // Disable global JIT state changes from Lua code.
    "disable_lua_load",      // Disable loadstring, dostring, etc from base lib.
    "lua_ext",               // Ext library such as lua_ext_tab_len, etc.
    "lua_load_no_bc",        // Treat all inputs as strings (no bytecode allowed).
    "windows_set_lib_names", // Allow setting LJLIBNAME and LJDLLNAME from environment variables.
    "lua_ext_ccatch_error"   // Throw lua errors which cannot be catched from lua standard
                             // pcall/xpcall but only from lua_pcall C API.
];

fn apply_patches(out_path: &Path) -> std::io::Result<Vec<String>> {
    Patch::new(
        &Path::new("..").join("patch"),
        &Path::new("..").join("LuaJIT"),
    )?
    .apply_all(PATCH_LIST.iter().copied(), out_path)
}

fn run_build(build_dir: &Path) -> std::io::Result<Lib> {
    let manifest = std::env::var_os("CARGO_MANIFEST_PATH")
        .map(PathBuf::from)
        .expect("Failed to read manifest path");
    let target_name = std::env::var("TARGET").expect("Failed to read build target");
    let base = BuildInfoBase {
        dynamic: DYNAMIC,
        target_name: &target_name,
        build_dir,
        manifest: &manifest,
    };
    BuildInfo::new(base)?.build()
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
    #[cfg(feature = "module")]
    println!(
        "cargo:rustc-env=RUSTC_VERSION={}",
        rustc_version::version().unwrap()
    );
}

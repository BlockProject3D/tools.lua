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

use std::path::Path;
use bp3d_lua::libs::files::chroot::{access, set_access, set_chroot, Permissions};
use bp3d_lua::libs::files::{Files, SandboxPath};
use bp3d_lua::libs::Lib;
use bp3d_lua::vm::RootVm;

#[test]
fn test_vm_files() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("/root")); // for testing only
    let path: SandboxPath = vm.run_code(c"return bp3d.files.Path.new('/myfile')").unwrap();
    let path = path.to_path(&vm).unwrap();
    assert_eq!(path, Path::new("/root/myfile"));
    let spath: SandboxPath = vm.run_code(c"return bp3d.files.Path.new('/data'):join('myfile'):withExtension('txt')").unwrap();
    let path = spath.to_path(&vm).unwrap();
    assert_eq!(path, Path::new("/root/data/myfile.txt"));
    let path = spath.to_str(&vm).unwrap();
    assert_eq!(path, "/data/myfile.txt");
    let path: SandboxPath = vm.run_code(c"return bp3d.files.Path.new('/')").unwrap();
    let path = path.to_path(&vm).unwrap();
    assert_eq!(path, Path::new("/root"));
}

#[test]
fn test_vm_files_security() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("/root")); // for testing only
    vm.run_code::<()>(c"return bp3d.files.Path.new('../myfile')").unwrap_err();
    vm.run_code::<()>(c"return bp3d.files.Path.new('/../myfile')").unwrap_err();
    vm.run_code::<()>(c"return bp3d.files.Path.new('/./../myfile')").unwrap_err();
    vm.run_code::<()>(c"return bp3d.files.Path.new('.././myfile/.')").unwrap_err();
    vm.run_code::<()>(c"return bp3d.files.Path.new('/data/../myfile')").unwrap();
    vm.run_code::<()>(c"return bp3d.files.Path.new('/data/../../myfile')").unwrap_err();
    vm.run_code::<()>(c"return bp3d.files.Path.new('/../data/myfile')").unwrap_err();
}

#[test]
fn test_vm_files_permissions() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("/root")); // for testing only
    set_access(&vm, "/rodata", Permissions::R);
    set_access(&vm, "/rwdata", Permissions::R | Permissions::W);
    set_access(&vm, "/data/myfile.lua", Permissions::R | Permissions::W | Permissions::X);
    assert_eq!(access(&vm, "/rodata/myfile.txt"), Permissions::R);
    assert_eq!(access(&vm, "/rodata.txt"), Permissions::NONE);
    assert_eq!(access(&vm, "/"), Permissions::NONE);
    assert_eq!(access(&vm, "/rodata"), Permissions::R);
    assert_eq!(access(&vm, "/rwdata"), Permissions::R | Permissions::W);
    assert_eq!(access(&vm, "/rwdata/myfile.txt"), Permissions::R | Permissions::W);
    assert_eq!(access(&vm, "/data"), Permissions::NONE);
    assert_eq!(access(&vm, "/data/myfile"), Permissions::NONE);
    assert_eq!(access(&vm, "/data/myfile.txt"), Permissions::NONE);
    assert_eq!(access(&vm, "/data/myfile.lua"), Permissions::R | Permissions::W | Permissions::X);
}

#[test]
fn test_vm_files_permissions2() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("/root")); // for testing only
    set_access(&vm, "/", Permissions::R);
    set_access(&vm, "/target", Permissions::R | Permissions::W);
    set_access(&vm, "/bp3d-build", Permissions::R | Permissions::X);
    assert_eq!(access(&vm, "/target/myfile"), Permissions::R | Permissions::W);
    assert_eq!(access(&vm, "/bp3d-build/myfile.lua"), Permissions::R | Permissions::X);
    assert_eq!(access(&vm, "/bp3d-build/package/myfile.lua"), Permissions::R | Permissions::X);
    assert_eq!(access(&vm, "/target/aarch64/data/1/myfile"), Permissions::R | Permissions::W);
    assert_eq!(access(&vm, "/myfile"), Permissions::R);
    assert_eq!(access(&vm, "/obj/data/1/myfile"), Permissions::R);
}

#[test]
fn test_vm_simple_chroot() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("."));
    let path: SandboxPath = vm.run_code(c"return bp3d.files.Path.new('/myfile')").unwrap();
    let path = path.to_path(&vm).unwrap();
    assert_eq!(path, Path::new("./myfile"));
}

#[test]
fn test_vm_simple_chroot2() {
    let vm = RootVm::new();
    Files.register(&vm).unwrap();
    set_chroot(&vm, Path::new("./"));
    let path: SandboxPath = vm.run_code(c"return bp3d.files.Path.new('/myfile')").unwrap();
    let path = path.to_path(&vm).unwrap();
    assert_eq!(path, Path::new("./myfile"));
}

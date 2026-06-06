// Copyright (c) 2026, BlockProject 3D
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

use crate::decl_lib_func;
use crate::libs::files::chroot::{access, sandbox, Permissions};
use crate::libs::files::SandboxPath;
use crate::vm::table::Table;
use bp3d_util::simple_error;
use std::fs::File;
use std::io::{ErrorKind, Read};

const MAX_FILE_SIZE: usize = 5000000; //5Mb

simple_error! {
    Error {
        Io(std::io::Error) => "io error: {}",
        Sandbox => "sandbox error",
        TooLarge(usize) => "file is too large ({})",
        Memory => "memory error",
        Permission => "permission denied",
        Unsupported => "unsupported operation"
    }
}

decl_lib_func! {
    pub fn read_text(vm: &Vm, path: SandboxPath) -> Result<String, Error> {
        if !(path.access(vm) & Permissions::R) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        let mut file = File::open(path).map_err(Error::Io)?;
        let size = file.metadata().map(|m| m.len() as usize).ok().unwrap_or(usize::MAX);
        if size > MAX_FILE_SIZE {
            return Err(Error::TooLarge(size));
        }
        let mut s = String::new();
        s.try_reserve_exact(size).map_err(|_| Error::Memory)?;
        file.read_to_string(&mut s).map_err(Error::Io)?;
        Ok(s)
    }
}

decl_lib_func! {
    pub fn write_text(vm: &Vm, path: SandboxPath, data: &str) -> Result<(), Error> {
        if !(path.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        std::fs::write(path, data).map_err(Error::Io)
    }
}

decl_lib_func! {
    pub fn copy_file(vm: &Vm, src_path: SandboxPath, dst_path: SandboxPath) -> Result<(), Error> {
        if !(src_path.access(vm) & Permissions::R) {
            return Err(Error::Permission);
        }
        if !(dst_path.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let src_path = src_path.to_path(vm).map_err(|_| Error::Sandbox)?;
        let dst_path = dst_path.to_path(vm).map_err(|_| Error::Sandbox)?;
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
        }
        std::fs::copy(src_path, dst_path).map(|_| ()).map_err(Error::Io)
    }
}

decl_lib_func! {
    pub fn symlink(vm: &Vm, src_path: SandboxPath, dst_path: SandboxPath) -> Result<(), Error> {
        #[cfg(unix)]
        {
            if src_path.is_relative() {
                let first_part = dst_path.to_path(vm).map_err(|_| Error::Sandbox)?;
                let path = first_part.join(src_path.as_os_str());
                let path = sandbox(vm, &path).map_err(|_| Error::Sandbox)?;
                if !(access(vm, &path) & Permissions::R) {
                    return Err(Error::Permission);
                }
            } else if !(src_path.access(vm) & Permissions::R) {
                return Err(Error::Permission);
            }
            if !(dst_path.access(vm) & Permissions::W) {
                return Err(Error::Permission);
            }
            let src_path = src_path.to_path(vm).map_err(|_| Error::Sandbox)?;
            let dst_path = dst_path.to_path(vm).map_err(|_| Error::Sandbox)?;
            return std::os::unix::fs::symlink(src_path, dst_path).map(|_| ()).map_err(Error::Io);
        }
        #[cfg(windows)]
        return Err(Error::Unsupported);
    }
}

decl_lib_func! {
    pub fn delete_dir(vm: &Vm, path: SandboxPath) -> Result<(), Error> {
        if !(path.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        std::fs::remove_dir_all(path).map_err(Error::Io)
    }
}

decl_lib_func! {
    pub fn create_dir(vm: &Vm, path: SandboxPath) -> Result<(), Error> {
        if !(path.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        std::fs::create_dir_all(path).map_err(Error::Io)
    }
}

decl_lib_func! {
    pub fn exists(vm: &Vm, path: SandboxPath) -> bool {
        path.to_path(vm).map(|v| v.exists()).unwrap_or(false)
    }
}

decl_lib_func! {
    pub fn list<'a>(vm: &Vm, path: SandboxPath) -> Result<Table<'a>, Error> {
        if !(path.access(vm) & Permissions::R) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        let mut tbl = Table::new(vm);
        let files = path.read_dir().map_err(Error::Io)?;
        for file in files {
            let file = file.map_err(Error::Io)?;
            let path = file.path();
            let name = file.file_name();
            let ty = file.file_type().map_err(Error::Io)?;
            let mut subt = Table::with_capacity(vm, 0, 4);
            subt.set(c"path", SandboxPath::from_path_unchecked(&path)).unwrap();
            subt.set(c"name", name.as_encoded_bytes()).unwrap();
            if ty.is_dir() {
                subt.set(c"type", "dir").unwrap();
            } else if ty.is_file() {
                subt.set(c"type", "file").unwrap();
            } else if ty.is_symlink() {
                subt.set(c"type", "symlink").unwrap();
            } else {
                subt.set(c"type", "other").unwrap();
            }
            tbl.push(subt).unwrap();
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    pub fn lua_access<'a>(vm: &Vm, path: SandboxPath) -> crate::vm::Result<Table<'a>> {
        let perms = path.access(vm);
        let mut tbl = Table::new(vm);
        if perms & Permissions::R {
            tbl.set(c"r", true)?;
        } else {
            tbl.set(c"r", false)?;
        }
        if perms & Permissions::W {
            tbl.set(c"w", true)?;
        } else {
            tbl.set(c"w", false)?;
        }
        if perms & Permissions::X {
            tbl.set(c"x", true)?;
        } else {
            tbl.set(c"x", false)?;
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    pub fn delete(vm: &Vm, path: SandboxPath) -> Result<(), Error> {
        if !(path.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        std::fs::remove_file(path).map_err(Error::Io)
    }
}

decl_lib_func! {
    pub fn rename(vm: &Vm, src: SandboxPath, dst: SandboxPath) -> Result<(), Error> {
        if !(src.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        if !(src.access(vm) & Permissions::R) {
            return Err(Error::Permission);
        }
        if !(dst.access(vm) & Permissions::W) {
            return Err(Error::Permission);
        }
        let src_path = src.to_path(vm).map_err(|_| Error::Sandbox)?;
        let dst_path = dst.to_path(vm).map_err(|_| Error::Sandbox)?;
        if !src_path.exists() {
            return Err(Error::Io(std::io::Error::new(ErrorKind::NotFound, "source file not found")));
        }
        if dst_path.exists() {
            return Err(Error::Io(std::io::Error::new(ErrorKind::AlreadyExists, "destination file already exists")));
        }
        std::fs::rename(src_path, dst_path).map_err(Error::Io)
    }
}

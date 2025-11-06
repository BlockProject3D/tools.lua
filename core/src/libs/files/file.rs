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

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use bp3d_util::simple_error;
use crate::{decl_lib_func, decl_userdata, impl_userdata_mut};
use crate::libs::files::chroot::Permissions;
use crate::libs::files::SandboxPath;
use crate::vm::value::types::UInt53;

const MAX_BUF_SIZE: usize = 65535;

simple_error! {
    Error {
        Io(std::io::Error) => "io error: {}",
        TooLarge(usize) => "internal buffer overflow ({})",
        Sandbox => "sandbox error",
        Permission => "permission denied"
    }
}

decl_userdata! {
    pub struct FileWrapper {
        file: File,
        is_ro: bool,
        buffer: [u8; MAX_BUF_SIZE]
    }
}

decl_lib_func! {
    fn open(vm: &Vm, path: SandboxPath, mode: &str) -> Result<FileWrapper, Error> {
        let perms = path.access(vm);
        let mut opts = OpenOptions::new();
        let mut is_ro = true;
        if mode.contains("r") {
            if !(perms & Permissions::R) {
                return Err(Error::Permission);
            }
            opts.read(true);
        }
        if mode.contains("w") {
            if !(perms & Permissions::W) {
                return Err(Error::Permission);
            }
            is_ro = false;
            opts.write(true);
        }
        if mode.contains("a") {
            if !(perms & Permissions::W) || !(perms & Permissions::R) {
                return Err(Error::Permission);
            }
            is_ro = false;
            opts.append(true);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        let file = opts.open(path).map_err(Error::Io)?;
        Ok(FileWrapper {
            file,
            is_ro,
            buffer: [0; MAX_BUF_SIZE]
        })
    }
}

decl_lib_func! {
    fn create(vm: &Vm, path: SandboxPath) -> Result<FileWrapper, Error> {
        let perms = path.access(vm);
        if !(perms & Permissions::W) {
            return Err(Error::Permission);
        }
        if !(perms & Permissions::R) {
            return Err(Error::Permission);
        }
        let path = path.to_path(vm).map_err(|_| Error::Sandbox)?;
        let file = File::create(path).map_err(Error::Io)?;
        Ok(FileWrapper {
            file,
            is_ro: false,
            buffer: [0; MAX_BUF_SIZE]
        })
    }
}

impl_userdata_mut! {
    impl FileWrapper {
        fn read(this: &mut FileWrapper, size: UInt53) -> Result<Option<&[u8]>, Error> {
            if size.to_usize() >= MAX_BUF_SIZE {
                return Err(Error::TooLarge(size.to_usize()));
            }
            let len = this.file.read(&mut this.buffer[..size.to_usize()]).map_err(Error::Io)?;
            if len == 0 {
                return Ok(None);
            }
            Ok(Some(&this.buffer[..len]))
        }

        fn write(this: &mut FileWrapper, buf: &[u8]) -> Result<UInt53, Error> {
            if this.is_ro {
                return Err(Error::Permission);
            }
            this.file.write(buf).map(UInt53::from_usize_lossy).map_err(Error::Io)
        }

        fn seek_from_start(this: &mut FileWrapper, pos: u64) -> std::io::Result<u64> {
            this.file.seek(std::io::SeekFrom::Start(pos))
        }

        fn seek_from_end(this: &mut FileWrapper, pos: i64) -> std::io::Result<u64> {
            this.file.seek(std::io::SeekFrom::End(pos))
        }

        fn seek_from_cursor(this: &mut FileWrapper, pos: i64) -> std::io::Result<u64> {
            this.file.seek(std::io::SeekFrom::Current(pos))
        }

        fn size(this: &FileWrapper) -> std::io::Result<u64> {
            this.file.metadata().map(|m| m.len())
        }
    }
    static {
        [fn open];
        [fn create];
    }
}

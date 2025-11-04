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

use std::borrow::Cow;
use std::path::PathBuf;
use crate::{decl_lib_func, decl_userdata, impl_userdata};
use crate::libs::files::chroot::{sandbox, unsandbox, SandboxError};
use crate::libs::files::SandboxPath;

decl_userdata!(pub struct PathWrapper(PathBuf));

impl PathWrapper {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }
}

decl_lib_func! {
    fn new(vm: &Vm, path: &str) -> Result<PathWrapper, SandboxError> {
        unsandbox(vm, path).map(|v| PathWrapper(v.into()))
    }
}

impl_userdata! {
    impl PathWrapper {
        fn join(this: &Path, vm: &Vm, other: SandboxPath) -> Result<PathWrapper, SandboxError> {
            let path = other.to_str(vm)?;
            Ok(PathWrapper(this.0.join(path.as_ref())))
        }

        fn with_extension(this: &Path, extension: &str) -> PathWrapper {
            PathWrapper(this.0.with_extension(extension))
        }

        fn with_name(this: &Path, name: &str) -> PathWrapper {
            let mut path = this.0.clone();
            path.set_file_name(name);
            PathWrapper(path)
        }

        fn name(this: &Path) -> Option<String> {
            this.0.file_name().map(|v| v.to_string_lossy().into())
        }

        fn extension(this: &Path) -> Option<String> {
            this.0.extension().map(|v| v.to_string_lossy().into())
        }

        fn __tostring<'a>(this: &Path, vm: &Vm) -> Cow<'a, str> {
            sandbox(vm, &this.0).unwrap_or(Cow::Borrowed("<sandbox error>"))
        }
    }
    static {
        [fn new];
    }
}

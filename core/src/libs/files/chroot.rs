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
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Display;
use std::ops::{BitAnd, BitOr};
use std::path::Path;
use bp3d_debug::trace;
use crate::vm::core::destructor::Pool;
use crate::vm::registry::named::Key;
use crate::vm::registry::types::LuaRef;
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;

const CHROOT: Key<LuaRef<&[u8]>> = Key::new("__chroot__");

pub fn with_chroot<R>(vm: &Vm, f: impl FnOnce(&Path) -> R) -> R {
    let s = CHROOT.push(vm);
    match s {
        None => f(Path::new("")),
        Some(v) => {
            let p = Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(v.get()) });
            f(p)
        }
    }
}

pub fn set_chroot(vm: &Vm, path: &Path) {
    let mut bytes = path.as_os_str().as_encoded_bytes();
    if bytes[bytes.len() - 1] == b'/' {
        bytes = &bytes[..bytes.len() - 1];
    }
    let r = crate::vm::registry::lua_ref::LuaRef::new(vm, bytes);
    CHROOT.set(r);
}

#[derive(Debug)]
pub struct SandboxError;

impl Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "attempt to escape the sandbox")
    }
}

impl std::error::Error for SandboxError {}

pub fn unsandbox<'a>(vm: &Vm, path: &'a str) -> Result<Cow<'a, Path>, SandboxError> {
    let mut level = 0;
    for component in path.split('/') {
        if component == ".." {
            if level == 0 {
                return Err(SandboxError);
            }
            level -= 2;
        } else if component == "." || component == "" {
        } else {
            level += 1;
        }
    }
    trace!({level}, "unsandbox {}", path);
    if level < 0 {
        return Err(SandboxError);
    }
    if path.len() > 0 && path.as_bytes()[0] == b'/' {
        return Ok(Cow::Owned(with_chroot(vm, |root| root.join(&path[1..]))))
    }
    Ok(Cow::Borrowed(Path::new(path)))
}

pub fn sandbox<'a>(vm: &Vm, path: &'a Path) -> Result<Cow<'a, str>, SandboxError> {
    let pos = with_chroot(vm, |root| {
        let src = path.as_os_str().as_encoded_bytes();
        let root = root.as_os_str().as_encoded_bytes();
        if !(src.len() >= root.len()) || !src.starts_with(root) {
            return 0;
        }
        root.len()
    });
    if pos == 0 {
        return Err(SandboxError)
    }
    let mut src = &path.as_os_str().as_encoded_bytes()[pos..];
    if src.len() == 0 {
        return Ok(Cow::Borrowed("/"));
    }
    if src[0] != b'/' {
        src = &path.as_os_str().as_encoded_bytes()[pos - 1..];
    }
    Ok(String::from_utf8_lossy(src))
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Permissions(u8);

impl Permissions {
    pub const R: Permissions = Permissions(0x1);
    pub const W: Permissions = Permissions(0x2);
    pub const X: Permissions = Permissions(0x4);

    pub const NONE: Permissions = Permissions(0x0);
}

impl BitOr for Permissions {
    type Output = Permissions;

    fn bitor(self, rhs: Self) -> Self::Output {
        Permissions(self.0 | rhs.0)
    }
}

impl BitAnd for Permissions {
    type Output = bool;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.0 & rhs.0 != 0
    }
}

const PERMISSIONS: Key<LuaRef<RawPtr<PermissionManager>>> = Key::new("__permissions__");

#[derive(Default)]
struct PermissionManager {
    permissions: HashMap<String, Permissions>,
}

impl PermissionManager {
    fn set_permissions(&mut self, path: &str, perms: Permissions) {
        self.permissions.insert(path.into(), perms);
    }

    fn get_permissions(&self, path: &str) -> Option<Permissions> {
        self.permissions.get(path).cloned()
    }

    fn set_permissions_vm(vm: &Vm, path: &str, permissions: Permissions) {
        let mut value = PERMISSIONS.push(vm);
        if value.is_none() {
            let ptr = Pool::attach_send(vm, Box::new(PermissionManager::default()));
            let r = crate::vm::registry::lua_ref::LuaRef::new(vm, RawPtr::new(ptr));
            PERMISSIONS.set(r);
            value = PERMISSIONS.push(vm);
        }
        let value = value.unwrap();
        unsafe { (&mut *value.get().as_mut_ptr()).set_permissions(path, permissions) };
    }
}

pub fn set_access(vm: &Vm, path: &str, perms: Permissions) {
    if !path.starts_with("/") {
        return;
    }
    PermissionManager::set_permissions_vm(vm, path, perms);
}

pub fn access(vm: &Vm, path: &str) -> Permissions {
    if !path.starts_with("/") {
        return Permissions::NONE;
    }
    let perms = PERMISSIONS.push(vm);
    if perms.is_none() {
        return Permissions::NONE;
    }
    let perms = perms.unwrap().get();
    let perms = unsafe { &*perms.as_ptr() };
    let mut path = path;
    while path.len() > 0 {
        if let Some(perms) = perms.get_permissions(path) {
            return perms;
        }
        let id = path.rfind('/');
        match id {
            Some(pos) => path = &path[..pos],
            None => break
        }
    }
    perms.get_permissions("/").unwrap_or(Permissions::NONE)
}

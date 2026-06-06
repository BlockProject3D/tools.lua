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

use crate::libs::files::chroot::{access, sandbox, unsandbox, Permissions, SandboxError};
use crate::libs::files::path::PathWrapper;
use crate::util::core::SimpleDrop;
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::util::LuaType;
use crate::vm::value::{FromLua, IntoLua};
use crate::vm::Vm;
use bp3d_debug::error;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub enum SandboxPath<'a> {
    String(&'a str),
    Path(&'a Path),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SandboxPathBuf {
    String(String),
    Path(PathBuf),
}

impl From<SandboxPath<'_>> for SandboxPathBuf {
    fn from(value: SandboxPath<'_>) -> Self {
        match value {
            SandboxPath::String(v) => SandboxPathBuf::String(v.into()),
            SandboxPath::Path(v) => SandboxPathBuf::Path(v.into()),
        }
    }
}

impl SandboxPathBuf {
    /// Creates a new [SandboxPathBuf] from an existing string.
    pub fn from_str(vm: &Vm, path: String) -> Result<SandboxPathBuf, SandboxError> {
        unsandbox(vm, &path)?;
        Ok(SandboxPathBuf::String(path))
    }

    /// Creates a new [SandboxPathBuf] from an existing string.
    ///
    /// If the given string cannot be represented in the current sandbox configuration, a nil value
    /// will be passed to Lua.
    pub fn from_str_unchecked(path: String) -> SandboxPathBuf {
        SandboxPathBuf::String(path)
    }

    /// Creates a new [SandboxPathBuf] from an existing [PathBuf].
    pub fn from_path<'a>(vm: &Vm, path: PathBuf) -> Result<SandboxPathBuf, SandboxError> {
        sandbox(vm, &path)?;
        Ok(SandboxPathBuf::Path(path))
    }

    /// Creates a new [SandboxPathBuf] from an existing [PathBuf].
    ///
    /// This function allows passing paths from Rust to Lua which are outside the sandbox.
    /// Use with caution.
    pub fn from_path_unchecked(path: PathBuf) -> SandboxPathBuf {
        SandboxPathBuf::Path(path)
    }

    pub fn as_path(&self) -> SandboxPath<'_> {
        match self {
            SandboxPathBuf::String(v) => SandboxPath::String(v),
            SandboxPathBuf::Path(v) => SandboxPath::Path(v),
        }
    }

    /// Returns the underlying path as raw [OsStr]. This function does not interpret the path
    /// according to the current sandbox configuration.
    pub fn as_os_str(&self) -> &OsStr {
        match self {
            SandboxPathBuf::String(v) => v.as_ref(),
            SandboxPathBuf::Path(v) => v.as_os_str(),
        }
    }
}

impl SandboxPath<'_> {
    /// Creates a new [SandboxPath] from an existing string.
    pub fn from_str<'a>(vm: &Vm, path: &'a str) -> Result<SandboxPath<'a>, SandboxError> {
        unsandbox(vm, path)?;
        Ok(SandboxPath::String(path))
    }

    /// Creates a new [SandboxPath] from an existing string.
    ///
    /// If the given string cannot be represented in the current sandbox configuration, a nil value
    /// will be passed to Lua.
    pub fn from_str_unchecked(path: &str) -> SandboxPath<'_> {
        SandboxPath::String(path)
    }

    /// Creates a new [SandboxPath] from an existing [Path].
    pub fn from_path<'a>(vm: &Vm, path: &'a Path) -> Result<SandboxPath<'a>, SandboxError> {
        sandbox(vm, path)?;
        Ok(SandboxPath::Path(path))
    }

    /// Creates a new [SandboxPath] from an existing [Path].
    ///
    /// This function allows passing paths from Rust to Lua which are outside the sandbox.
    /// Use with caution.
    pub fn from_path_unchecked(path: &Path) -> SandboxPath<'_> {
        SandboxPath::Path(path)
    }

    /// Returns the underlying path as raw [OsStr]. This function does not interpret the path
    /// according to the current sandbox configuration.
    pub fn as_os_str(&self) -> &OsStr {
        match self {
            SandboxPath::String(v) => v.as_ref(),
            SandboxPath::Path(v) => v.as_os_str(),
        }
    }

    pub fn to_str(&self, vm: &Vm) -> Result<Cow<'_, str>, SandboxError> {
        match self {
            SandboxPath::String(v) => {
                unsandbox(vm, v)?;
                Ok(Cow::Borrowed(v))
            }
            SandboxPath::Path(v) => sandbox(vm, v),
        }
    }

    pub fn to_path(&self, vm: &Vm) -> Result<Cow<'_, Path>, SandboxError> {
        match self {
            SandboxPath::String(v) => unsandbox(vm, *v),
            SandboxPath::Path(v) => Ok(Cow::Borrowed(v)),
        }
    }

    pub fn access(&self, vm: &Vm) -> Permissions {
        let path = match self.to_str(vm) {
            Ok(v) => v,
            Err(e) => {
                error!("failed to read permissions for {:?}: {}", self, e);
                return Permissions::NONE;
            }
        };
        access(vm, &path)
    }

    pub fn is_relative(&self) -> bool {
        match self {
            SandboxPath::String(v) => !v.starts_with("/"),
            SandboxPath::Path(v) => !v.starts_with("/"),
        }
    }
}

impl<'a> FromLua<'a> for SandboxPath<'a> {
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self {
        let wrapper: crate::vm::Result<&PathWrapper> = FromLua::from_lua(vm, index);
        if let Ok(wrapper) = wrapper {
            return SandboxPath::Path(wrapper.path());
        }
        let s: &str = FromLua::from_lua_unchecked(vm, index);
        SandboxPath::String(s)
    }

    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self> {
        let wrapper: crate::vm::Result<&PathWrapper> = FromLua::from_lua(vm, index);
        if let Ok(wrapper) = wrapper {
            return Ok(SandboxPath::Path(wrapper.path()));
        }
        let s: &str = FromLua::from_lua(vm, index)?;
        Ok(SandboxPath::String(s))
    }
}

impl FromLua<'_> for SandboxPathBuf {
    unsafe fn from_lua_unchecked(vm: &'_ Vm, index: i32) -> Self {
        SandboxPath::from_lua_unchecked(vm, index).into()
    }

    fn from_lua(vm: &'_ Vm, index: i32) -> crate::vm::Result<Self> {
        SandboxPath::from_lua(vm, index).map(SandboxPathBuf::from)
    }
}

unsafe impl IntoLua for SandboxPath<'_> {
    fn into_lua(self, vm: &Vm) -> u16 {
        match self {
            SandboxPath::String(v) => unsandbox(vm, v)
                .map(|path| PathWrapper::new(path.into()))
                .ok()
                .into_lua(vm),
            SandboxPath::Path(v) => PathWrapper::new(v.into()).into_lua(vm),
        }
    }
}

unsafe impl SimpleDrop for SandboxPath<'_> {}

impl LuaType for SandboxPath<'_> {}

impl<'a> FromParam<'a> for SandboxPath<'a> {
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self {
        let wrapper: Option<&PathWrapper> = FromParam::try_from_param(vm, index);
        if let Some(wrapper) = wrapper {
            return SandboxPath::Path(wrapper.path());
        }
        let s: &str = FromParam::from_param(vm, index);
        SandboxPath::String(s)
    }

    fn try_from_param(vm: &'a Vm, index: i32) -> Option<Self> {
        let wrapper: Option<&PathWrapper> = FromParam::try_from_param(vm, index);
        if let Some(wrapper) = wrapper {
            return Some(SandboxPath::Path(wrapper.path()));
        }
        let wrapper: Option<&str> = FromParam::try_from_param(vm, index);
        wrapper.map(|v| SandboxPath::String(v))
    }
}

unsafe impl IntoParam for SandboxPath<'_> {
    #[inline(always)]
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self, vm) as _
    }
}

unsafe impl IntoParam for SandboxPathBuf {
    fn into_param(self, vm: &Vm) -> i32 {
        IntoLua::into_lua(self.as_path(), vm) as _
    }
}

unsafe impl IntoLua for SandboxPathBuf {
    fn into_lua(self, vm: &Vm) -> u16 {
        IntoLua::into_lua(self.as_path(), vm)
    }
}

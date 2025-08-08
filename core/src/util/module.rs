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

use crate::module::error::ErrorType;
use crate::module::{TIME_VERSION, VERSION};
use crate::vm::error::{RuntimeError, TypeError, Utf8Error};
use crate::vm::Vm;
use bp3d_debug::info;
use bp3d_os::module::library::types::VirtualLibrary;
use bp3d_os::module::library::Library;
use bp3d_os::module::{Module, ModuleLoader};
use bp3d_util::simple_error;
use std::collections::HashSet;
use std::ffi::CStr;
use std::path::PathBuf;

simple_error! {
    pub Error {
        PluginAlreadyLoaded(String) => "plugin {} is already loaded",
        LibNotFound(String) => "library not found: {}",
        PluginNotFound(String) => "plugin not found: {}",
        (impl From)Vm(crate::vm::error::Error) => "vm error: {}",
        (impl From)Module(bp3d_os::module::error::Error) => "module error: {}"
    }
}

pub type Result<T> = std::result::Result<T, Error>;

type PluginFunc =
    extern "C" fn(l: crate::ffi::lua::State, error: *mut crate::module::error::Error) -> bool;

unsafe fn get_string(
    err: &crate::module::error::Error,
    f: impl FnOnce(String) -> crate::vm::error::Error,
) -> crate::vm::error::Error {
    match std::str::from_utf8(&err.string.data[..err.string.len]) {
        Ok(v) => f(v.into()),
        Err(e) => crate::vm::error::Error::InvalidUtf8(e.into()),
    }
}

unsafe fn convert_module_error_to_vm_error(
    err: crate::module::error::Error,
) -> crate::vm::error::Error {
    match err.ty {
        ErrorType::Utf8 => crate::vm::error::Error::InvalidUtf8(Utf8Error {
            valid_up_to: err.utf8.valid_up_to,
            error_len: if err.utf8.error_len < 0 {
                None
            } else {
                Some(err.utf8.error_len as u8)
            },
        }),
        ErrorType::Type => crate::vm::error::Error::Type(TypeError {
            expected: err.type_mismatch.expected,
            actual: err.type_mismatch.actual,
        }),
        ErrorType::Syntax => get_string(&err, crate::vm::error::Error::Syntax),
        ErrorType::Runtime => get_string(&err, |v| {
            crate::vm::error::Error::Runtime(RuntimeError::new(v))
        }),
        ErrorType::UncatchableRuntime => get_string(&err, |v| {
            crate::vm::error::Error::UncatchableRuntime(RuntimeError::new(v))
        }),
        ErrorType::Memory => crate::vm::error::Error::Memory,
        ErrorType::Unknown => crate::vm::error::Error::Unknown,
        ErrorType::Error => crate::vm::error::Error::Error,
        ErrorType::Null => crate::vm::error::Error::Null,
        ErrorType::MultiValue => crate::vm::error::Error::MultiValue,
        ErrorType::UnsupportedType => {
            crate::vm::error::Error::UnsupportedType(err.unsupported_type.actual)
        }
        ErrorType::Loader => get_string(&err, crate::vm::error::Error::Loader),
        ErrorType::ParseFloat => crate::vm::error::Error::ParseFloat,
        ErrorType::ParseInt => crate::vm::error::Error::ParseInt,
        ErrorType::UserDataArgsEmpty => {
            crate::vm::error::Error::UserData(crate::vm::userdata::Error::ArgsEmpty)
        }
        ErrorType::UserDataMutViolation => crate::vm::error::Error::UserData(
            crate::vm::userdata::Error::MutViolation(CStr::from_ptr(err.static_string.data)),
        ),
        ErrorType::UserDataGc => crate::vm::error::Error::UserData(crate::vm::userdata::Error::Gc),
        ErrorType::UserDataIndex => {
            crate::vm::error::Error::UserData(crate::vm::userdata::Error::Index)
        }
        ErrorType::UserDataMetatable => {
            crate::vm::error::Error::UserData(crate::vm::userdata::Error::Metatable)
        }
        ErrorType::UserDataMultiValueField => {
            crate::vm::error::Error::UserData(crate::vm::userdata::Error::MultiValueField)
        }
        ErrorType::UserDataAlreadyRegistered => crate::vm::error::Error::UserData(
            crate::vm::userdata::Error::AlreadyRegistered(CStr::from_ptr(err.static_string.data)),
        ),
        ErrorType::UserDataAlignment => crate::vm::error::Error::UserData(
            crate::vm::userdata::Error::Alignment(err.alignment.alignment),
        ),
        ErrorType::None => std::hint::unreachable_unchecked(),
        ErrorType::BadThreadState => crate::vm::error::Error::BadThreadState
    }
}

pub struct ModuleManager {
    set: HashSet<String>,
    loader: ModuleLoader,
}

// This is safe because ModuleManager does not use thread locals or mutable globals of some kind.
// The hard work is mostly done by syscalls or static read-only memory.
unsafe impl Send for ModuleManager {}

impl ModuleManager {
    fn load_plugin<L: Library>(
        vm: &Vm,
        module: &Module<L>,
        name: &str,
        lib: &str,
        plugin: &str,
    ) -> Result<()> {
        let func_name = format!("bp3d_lua_{}_register_{}", lib, plugin);
        let sym = unsafe { module.lib().load_symbol::<PluginFunc>(func_name) }?
            .ok_or_else(|| Error::PluginNotFound(name.into()))?;
        let mut err = crate::module::error::Error {
            ty: ErrorType::None,
        };
        if !sym.call(vm.as_ptr(), &mut err) {
            return Err(Error::Vm(unsafe { convert_module_error_to_vm_error(err) }));
        }
        Ok(())
    }

    fn load_dynamic(&mut self, lib: &str, plugin: &str, vm: &Vm) -> Result<()> {
        let name = format!("{}::{}", lib, plugin);
        if self.set.contains(&name) {
            return Err(Error::PluginAlreadyLoaded(name));
        }
        let module = unsafe { self.loader.load(lib) }?;
        info!(
            "Loaded dynamic module {:?}-{:?}",
            module.get_metadata_key("NAME"),
            module.get_metadata_key("VERSION")
        );
        Self::load_plugin(vm, module, &name, &lib.replace("-", "_"), plugin)?;
        info!("Loaded plugin {}", name);
        self.set.insert(name);
        Ok(())
    }

    fn load_builtin(&mut self, lib: &str, plugin: &str, vm: &Vm) -> Result<bool> {
        let name = format!("{}::{}", lib, plugin);
        if self.set.contains(&name) {
            return Err(Error::PluginAlreadyLoaded(name));
        }
        let module = match unsafe { self.loader.load_builtin(lib) } {
            Ok(v) => v,
            Err(e) => {
                return match e {
                    bp3d_os::module::error::Error::NotFound(_) => Ok(false),
                    e => Err(Error::Module(e)),
                }
            }
        };
        info!(
            "Loaded builtin module {:?}-{:?}",
            module.get_metadata_key("NAME"),
            module.get_metadata_key("VERSION")
        );
        Self::load_plugin(vm, module, &name, &lib.replace("-", "_"), plugin)?;
        info!("Loaded plugin {}", name);
        self.set.insert(name);
        Ok(true)
    }

    pub fn load(&mut self, lib: &str, plugin: &str, vm: &Vm) -> Result<()> {
        if !self.load_builtin(lib, plugin, vm)? {
            self.load_dynamic(lib, plugin, vm)?;
        }
        Ok(())
    }

    pub fn add_search_path(&mut self, name: PathBuf) {
        self.loader.add_search_path(name)
    }

    pub fn new(builtins: &'static [&'static VirtualLibrary]) -> Self {
        let mut loader = ModuleLoader::new(builtins);
        loader.add_public_dependency("bp3d-lua", VERSION);
        loader.add_public_dependency("time", TIME_VERSION);
        Self {
            set: Default::default(),
            loader,
        }
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new(&[])
    }
}

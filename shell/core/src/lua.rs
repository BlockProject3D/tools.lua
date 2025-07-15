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

use std::path::PathBuf;
use tokio::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use bp3d_lua::vm::core::interrupt::{spawn_interruptible, Signal};
use bp3d_lua::libs;
use bp3d_lua::libs::Lib;
use bp3d_debug::{debug, error, info};
use bp3d_lua::vm::core::jit::JitOptions;
use bp3d_lua::vm::core::load::{Code, Script};
use bp3d_lua::vm::value::any::AnyValue;
use bp3d_lua::vm::Vm;
use crate::autocomplete::Autocomplete;
use crate::data::DataOut;
use crate::data_in::{Exit, InData, NetInData, RunCode, RunFile};
use crate::data_out::{Log, OutData};

const CHANNEL_BUFFER: usize = 32;

pub struct Args {
    pub data: PathBuf,
    pub lua: PathBuf,
    pub modules: Vec<PathBuf>
}

pub struct Lua {
    signal: Signal,
    handle: JoinHandle<()>,
    exec_queue: mpsc::Sender<Box<dyn InData>>,
    out_queue: mpsc::Receiver<Box<dyn OutData>>,
}

impl Lua {
    pub async fn send<T: NetInData>(&self, net_data: T) {
        let data = net_data.to_in_data();
        self.exec_queue.send(data).await.unwrap();
    }

    pub async fn exit(self) {
        self.exec_queue.send(Box::new(Exit)).await.unwrap();
        // Leave 50ms for the thread to terminate nominally before killing the VM.
        tokio::time::sleep(Duration::from_millis(50)).await;
        // This call will either immediately return because the thread is already dead (expected),
        // otherwise it will pthread_kill and attempt to inject a lua hook which will later throw
        // a lua uncatchable error which should bring the VM down.
        let res = self.signal.send(Duration::from_secs(10));
        match res {
            Ok(_) => self.handle.join().unwrap(),
            Err(e) => error!("Error attempting to terminate VM thread: {}", e)
        }
    }

    pub async fn next_msg(&mut self) -> Option<Box<dyn OutData>> {
        self.out_queue.recv().await
    }

    fn handle_value(res: bp3d_lua::vm::Result<AnyValue>, logger: &DataOut) -> bool {
        match res {
            Ok(v) => {
                logger.send(Log("output", v.to_string()));
                false
            },
            Err(e) => {
                if e.is_uncatchable() {
                    logger.send(Log("kill", e.to_string()));
                    error!("Received VM termination error: {}", e);
                    true
                } else {
                    logger.send(Log("error", e.to_string()));
                    error!("Failed to run code: {}", e);
                    false
                }
            }
        }
    }

    pub fn new(args: Args) -> Self {
        let (exec_queue, mut receiver) = mpsc::channel(CHANNEL_BUFFER);
        let (logger, out_queue) = mpsc::channel(CHANNEL_BUFFER);
        let (signal, handle) = spawn_interruptible(move |vm| {
            let logger = DataOut::new(logger);
            debug!("Loading VM libraries...");
            if let Err(e) = (libs::os::Compat, libs::os::Instant, libs::os::Time).register(vm) {
                error!("Failed to load OS library: {}", e);
            }
            if let Err(e) = (libs::util::String, libs::util::Table, libs::util::Utf8).register(vm) {
                error!("Failed to load util library: {}", e);
            }
            if let Err(e) = libs::lua::Lua::new().load_chroot_path(&args.data).build().register(vm) {
                error!("Failed to load base library: {}", e);
            }
            if let Err(e) = Autocomplete::new(logger.clone()).register(vm) {
                error!("Failed to register autocomplete library: {}", e);
            }
            let mut modules = libs::lua::Module::new(&[]);
            for path in &args.modules {
                modules.add_search_path(path.clone());
            }
            if let Err(e) = modules.register(vm) {
                error!("Failed to load module manager: {}", e);
            }
            let jit = JitOptions::get(vm);
            if jit.is_enabled() {
                info!("JIT: ON ({})", jit.opt_level());
                info!("{}", jit.opts());
                info!("{}", jit.cpu());
            } else {
                info!("JIT: OFF")
            }
            while let Some(command) = receiver.blocking_recv() {
                // Nice type-inference breakage with this box.
                let ret = vm.scope(|vm| Ok((command as Box<dyn InData>).handle(&args, vm, &logger))).unwrap();
                if ret {
                    break;
                }
            }
        });
        Self {
            signal,
            handle,
            exec_queue,
            out_queue
        }
    }
}

impl InData for RunCode {
    fn handle(&mut self, _: &Args, vm: &Vm, out: &DataOut) -> bool {
        match &self.name {
            Some(name) => Lua::handle_value(vm.run(Code::new(name, self.code.as_bytes())), out),
            None => Lua::handle_value(vm.run_code(&*self.code), out)
        }
    }
}

impl InData for RunFile {
    fn handle(&mut self, args: &Args, vm: &Vm, out: &DataOut) -> bool {
        let path = args.lua.join(&self.path);
        let script = match Script::from_path(path) {
            Ok(script) => script,
            Err(e) => {
                error!("Error loading lua script: {}", e);
                out.send(Log("file", e.to_string()));
                return false;
            }
        };
        Lua::handle_value(vm.run(script), out)
    }
}

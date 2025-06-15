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

const CHANNEL_BUFFER: usize = 32;

pub struct Args {
    pub data: PathBuf,
    pub lua: PathBuf,
    pub modules: Vec<PathBuf>
}

pub enum Command {
    Exit,
    RunCode(String),
    RunCodeWithName(String, String),
    RunFile(String)
}

pub struct Lua {
    signal: Signal,
    handle: JoinHandle<()>,
    exec_queue: mpsc::Sender<Command>,
    log_queue: mpsc::Receiver<(&'static str, String)>,
}

impl Lua {
    pub async fn exec(&self, code: String) {
        self.exec_queue.send(Command::RunCode(code)).await.unwrap();
    }

    pub async fn exec_file(&self, name: String) {
        self.exec_queue.send(Command::RunFile(name)).await.unwrap();
    }

    pub async fn exec_with_name(&self, name: String, code: String) {
        self.exec_queue.send(Command::RunCodeWithName(name, code)).await.unwrap();
    }

    pub async fn exit(self) {
        self.exec_queue.send(Command::Exit).await.unwrap();
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

    pub async fn next_log(&mut self) -> Option<(&'static str, String)> {
        self.log_queue.recv().await
    }

    fn handle_value(res: bp3d_lua::vm::Result<AnyValue>, logger: &mpsc::Sender<(&'static str, String)>) -> bool {
        match res {
            Ok(v) => {
                logger.blocking_send(("output", v.to_string())).unwrap();
                false
            },
            Err(e) => {
                if e.is_uncatchable() {
                    logger.blocking_send(("kill", e.to_string())).unwrap();
                    error!("Received VM termination error: {}", e);
                    true
                } else {
                    logger.blocking_send(("error", e.to_string())).unwrap();
                    error!("Failed to run code: {}", e);
                    false
                }
            }
        }
    }

    pub fn new(args: Args) -> Self {
        let (exec_queue, mut receiver) = mpsc::channel(CHANNEL_BUFFER);
        let (logger, log_queue) = mpsc::channel(CHANNEL_BUFFER);
        let (signal, handle) = spawn_interruptible(move |vm| {
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
            let mut modules = libs::lua::Module::new(&[]);
            for path in args.modules {
                modules.add_search_path(path);
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
                match command {
                    Command::Exit => break,
                    Command::RunCode(code) => {
                        let ret = vm.scope(|vm| Ok(Self::handle_value(vm.run_code(&*code), &logger))).unwrap();
                        if ret {
                            break;
                        }
                    },
                    Command::RunCodeWithName(name, code) => {
                        let ret = vm.scope(|vm| Ok(Self::handle_value(vm.run(Code::new(&name, code.as_bytes())), &logger))).unwrap();
                        if ret {
                            break;
                        }
                    },
                    Command::RunFile(name) => {
                        let path = args.lua.join(name);
                        let script = match Script::from_path(path) {
                            Ok(script) => script,
                            Err(e) => {
                                error!("Error loading lua script: {}", e);
                                logger.blocking_send(("file", e.to_string())).unwrap();
                                continue;
                            }
                        };
                        let ret = vm.scope(|vm| Ok(Self::handle_value(vm.run(script), &logger))).unwrap();
                        if ret {
                            break;
                        }
                    }
                }
            }
        });
        Self {
            signal,
            handle,
            exec_queue,
            log_queue
        }
    }
}

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
use clap::Parser;

mod lua;
mod core;
mod autocomplete;
mod data_out;
mod data_in;
mod data;
mod scheduler;
mod scheduler_api;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'n', long = "name", help = "Name of IPC server.")]
    pub name: Option<String>,

    #[arg(short = 'r', long = "root", help = "Path to lua root directory.")]
    pub root: Option<PathBuf>,

    #[arg(short = 'm', long = "modules", help = "Path to modules directory.")]
    pub modules: Option<PathBuf>,

    #[arg(long = "it", help = "Run in interactive mode.")]
    pub interactive: bool,

    #[arg(help = "Path to main script to start at Vm startup in the root directory.")]
    pub main_script: Option<String>
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let root = args.root.unwrap_or(PathBuf::from("./"));
    let mut modules = Vec::new();
    if let Some(path) = args.modules {
        modules.push(path);
    }
    modules.push(PathBuf::from("./target/debug"));
    let largs = lua::Args {
        data: root.join("data"),
        lua: root.join("src"),
        modules,
        main_script: args.main_script,
    };
    if args.interactive {
        core::run_interactive(largs).await;
    } else {
        core::run(largs, args.name.as_ref().map(|v| &**v).unwrap_or("bp3d-lua-shell")).await;
    }
}

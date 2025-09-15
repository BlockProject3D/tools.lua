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

const MAX_SIZE: usize = 4096;

use crate::lua::{Args, Lua};
use bp3d_debug::{debug, error, info};
use bp3d_lua_shell_proto::send;
use bp3d_net::ipc::util::Message;
use bp3d_net::ipc::{Client, Server};
use bp3d_os::shell::{Event, SendChannel, Shell};
use bp3d_proto::message::FromBytes;
use bp3d_util::result::ResultExt;
use tokio::sync::mpsc;

async fn client_task(lua: &mut Lua, client: Client) -> bp3d_proto::message::Result<bool> {
    let mut msg = Message::new(MAX_SIZE);
    loop {
        tokio::select! {
            res = client.recv(&mut msg) => {
                res?;
                if msg.is_empty() {
                    break;
                }
                let data: &[u8] = &msg;
                //Nice weird broken syntax because Rust type inference is even more broken than ever.
                let msg = <send::Main>::from_bytes(data)?.into_inner();
                match msg.msg {
                    send::Message::Terminate => return Ok(true),
                    send::Message::RunCode(v) => lua.send(v).await,
                    send::Message::RunFile(v) => lua.send(v).await
                }
            },
            Some(b) = lua.next_msg() => {
                msg.set_size(0);
                b.write(&mut msg)?;
                client.send(&msg).await?;
            }
        }
    }
    client.close().await?;
    Ok(false)
}

pub async fn run(args: Args, name: &str) {
    info!("starting lua VM");
    let mut lua = Lua::new(args);
    info!("starting IPC server");
    let mut server = Server::create(name)
        .await
        .expect_exit("Failed to create IPC server", 1);
    while let Ok(client) = server.accept().await {
        debug!("client connected");
        match client_task(&mut lua, client).await {
            Err(e) => error!("client message error: {}", e),
            Ok(flag) => {
                debug!("client nominal exit");
                if flag {
                    break;
                }
            }
        }
    }
    info!("terminating lua VM...");
    lua.exit().await;
}

struct ChannelWrapper(mpsc::Sender<Event>);

impl SendChannel for ChannelWrapper {
    fn send(&self, event: Event) {
        self.0.blocking_send(event).unwrap();
    }
}

pub async fn run_interactive(args: Args) {
    info!("starting lua VM");
    let mut lua = Lua::new(args);
    let (tx, mut rx) = mpsc::channel::<Event>(64);
    let app = Shell::new("lua> ", ChannelWrapper(tx));
    loop {
        tokio::select! {
            res = rx.recv() => {
                match res {
                    None => {
                        error!("Shell application has prematurely closed");
                        break;
                    },
                    Some(event) => match event {
                        Event::CommandReceived(str) => {
                            lua.send(send::RunCode {
                                name: None,
                                code: &str
                            }).await;
                        },
                        Event::ExitRequested => {
                            debug!("exit requested");
                            break;
                        }
                    }
                }
            }
            Some(b) = lua.next_msg() => {
                debug!("{:?}", b);
                if b.has_exited() {
                    break;
                }
            }
        }
    }
    info!("Terminating lua VM...");
    lua.exit().await;
    info!("Terminating shell application...");
    app.exit();
}

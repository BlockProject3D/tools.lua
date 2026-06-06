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

use crate::autocomplete::{Mode, Type};
use bp3d_lua_shell_proto::completion;
use bp3d_lua_shell_proto::recv;
use bp3d_net::ipc::util::Message;
use bp3d_proto::message::WriteSelf;
use std::fmt::Debug;

pub trait OutData: Send + Debug {
    fn write(&self, msg: &mut Message) -> bp3d_proto::message::Result<()>;
    fn has_exited(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct End;

impl OutData for End {
    fn write(&self, msg: &mut Message) -> bp3d_proto::message::Result<()> {
        recv::Main {
            hdr: recv::Header::new().set_type(recv::Type::End).to_ref(),
            msg: recv::Message::End,
        }
        .write_self(msg)
    }

    fn has_exited(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Log(pub &'static str, pub String);

impl OutData for Log {
    fn write(&self, msg: &mut Message) -> bp3d_proto::message::Result<()> {
        recv::Main {
            hdr: recv::Header::new().set_type(recv::Type::Log).to_ref(),
            msg: recv::Log {
                source: self.0,
                msg: &self.1,
            },
        }
        .write_self(msg)
    }
}

#[derive(Debug)]
pub struct Autocomplete(pub Mode);

impl OutData for Autocomplete {
    fn write(&self, msg: &mut Message) -> bp3d_proto::message::Result<()> {
        match &self.0 {
            Mode::AddUpdate(v) => {
                let mut items = completion::AddUpdateItems::new(Vec::<u8>::new());
                for completion in v {
                    let mut items2 = completion::ListItems::new(Vec::<u8>::new());
                    for item in &completion.items {
                        let ty = match item.ty {
                            Type::Function => completion::Type::Function,
                            Type::Attribute => completion::Type::Attribute,
                        };
                        items2.write_item(&completion::Item {
                            hdr: completion::Header::new().set_type(ty).to_ref(),
                            name: &item.name,
                        })?;
                    }
                    items.write_item(&completion::List {
                        path: &completion.path,
                        items: items2.to_ref(),
                    })?;
                }
                recv::Main {
                    hdr: recv::Header::new()
                        .set_type(recv::Type::AutocompleteAddUpdate)
                        .to_ref(),
                    msg: completion::AddUpdate {
                        items: items.to_ref(),
                    },
                }
                .write_self(msg)?;
            }
            Mode::Delete(v) => {
                let mut items = completion::DeleteItems::new(Vec::<u8>::new());
                for path in v {
                    items.write_item(&completion::Path { path })?;
                }
                recv::Main {
                    hdr: recv::Header::new()
                        .set_type(recv::Type::AutocompleteDelete)
                        .to_ref(),
                    msg: completion::Delete {
                        items: items.to_ref(),
                    },
                }
                .write_self(msg)?;
            }
        }
        Ok(())
    }
}

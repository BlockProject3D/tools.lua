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

use bp3d_lua::vm::Vm;
use crate::data::DataOut;
use crate::lua::Args;

pub trait InData: Send {
    fn handle(&mut self, args: &Args, vm: &Vm, out: &DataOut) -> bool;
}

pub trait NetInData {
    fn to_in_data(self) -> Box<dyn InData>;
}

pub struct RunCode {
    pub name: Option<String>,
    pub code: String,
}

pub struct RunFile {
    pub path: String,
}

impl<'a> NetInData for bp3d_lua_shell_proto::send::RunFile<'a> {
    fn to_in_data(self) -> Box<dyn InData> {
        Box::new(RunFile {
            path: self.path.into()
        })
    }
}

impl<'a> NetInData for bp3d_lua_shell_proto::send::RunCode<'a> {
    fn to_in_data(self) -> Box<dyn InData> {
        Box::new(RunCode {
            name: self.name.map(|v| v.into()),
            code: self.code.into()
        })
    }
}

pub struct Exit;

impl InData for Exit {
    fn handle(&mut self, _: &Args, _: &Vm, _: &DataOut) -> bool {
        true
    }
}

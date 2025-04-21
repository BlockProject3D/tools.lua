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

mod context;
mod context_opt;

use bp3d_lua::decl_lib_func;
use bp3d_lua::vm::function::types::RFunction;
use bp3d_lua::vm::RootVm;
use mlua::Lua;
use std::time::Duration;

struct ValueWithDrop;
impl ValueWithDrop {
    pub fn print(&self) {}
}
impl Drop for ValueWithDrop {
    fn drop(&mut self) {}
}

decl_lib_func! {
    fn test_c_function(name: &str, value: f64) -> String {
        let drop = ValueWithDrop;
        drop.print();
        format!("Hello {} ({})", name, value)
    }
}

fn test_vm_destructor() -> Duration {
    let mut vm = RootVm::new();
    vm.set_global(c"test_c_function", RFunction::wrap(test_c_function))
        .unwrap();
    let time = bp3d_os::time::Instant::now();
    for _ in 0..20000 {
        let res = vm.run_code::<&str>(c"return test_c_function('this is a test\\xFF', 0.42)");
        assert!(res.is_err());
        assert!(vm
            .run_code::<&str>(c"return test_c_function('this is a test', 0.42)")
            .is_ok());
        let s = vm
            .run_code::<&str>(c"return test_c_function('this is a test', 0.42)")
            .unwrap();
        assert_eq!(s, "Hello this is a test (0.42)");
        vm.clear();
    }
    let time = time.elapsed();
    time
}

fn test_vm_mlua() -> Duration {
    let lua = Lua::new();
    let f = lua
        .create_function(|_, (name, value): (String, f64)| {
            let drop = ValueWithDrop;
            drop.print();
            Ok(format!("Hello {} ({})", name, value))
        })
        .unwrap();
    lua.globals().set("test_c_function", f).unwrap();
    let time = bp3d_os::time::Instant::now();
    for _ in 0..20000 {
        let res: mlua::Result<String> = lua
            .load("return test_c_function('this is a test\\xFF', 0.42)")
            .call(());
        assert!(res.is_err());
        assert!(lua
            .load("return test_c_function('this is a test', 0.42)")
            .call::<String>(())
            .is_ok());
        let s: String = lua
            .load("return test_c_function('this is a test', 0.42)")
            .call(())
            .unwrap();
        assert_eq!(s, "Hello this is a test (0.42)");
    }
    let time = time.elapsed();
    time
}

fn main() {
    const RUNS: u32 = 10;
    let mut lua = Duration::new(0, 0);
    let mut mlua = Duration::new(0, 0);
    let mut ctx_lua = Duration::new(0, 0);
    let mut ctx_mlua = Duration::new(0, 0);
    let mut ctx_lua_opt = Duration::new(0, 0);
    let mut ctx_mlua_opt = Duration::new(0, 0);

    for _ in 0..RUNS {
        lua += test_vm_destructor();
        mlua += test_vm_mlua();
        ctx_lua += context::test_context_vm();
        ctx_mlua += context::test_context_mlua();
        ctx_lua_opt += context_opt::test_context_vm();
        ctx_mlua_opt += context_opt::test_context_mlua();
    }

    lua = lua / RUNS;
    mlua = mlua / RUNS;
    ctx_lua = ctx_lua / RUNS;
    ctx_mlua = ctx_mlua / RUNS;
    ctx_lua_opt = ctx_lua_opt / RUNS;
    ctx_mlua_opt = ctx_mlua_opt / RUNS;

    println!("average tools.lua (basic): {:?}", lua);
    println!("average mlua (basic): {:?}", mlua);
    assert!(lua < mlua);
    println!("average diff (basic): {:?}", mlua - lua);

    println!("average tools.lua (context): {:?}", ctx_lua);
    println!("average mlua (context): {:?}", ctx_mlua);
    assert!(ctx_lua < ctx_mlua);
    println!("average diff (context): {:?}", ctx_mlua - ctx_lua);

    println!("average tools.lua (context_opt): {:?}", ctx_lua_opt);
    println!("average mlua (context_opt): {:?}", ctx_mlua_opt);
    assert!(ctx_lua_opt < ctx_mlua_opt);
    println!(
        "average diff (context_opt): {:?}",
        ctx_mlua_opt - ctx_lua_opt
    );
}

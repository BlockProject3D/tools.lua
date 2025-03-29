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

use std::time::Duration;
use mlua::{Function, Lua, UserDataMethods};
use bp3d_lua::decl_closure;
use bp3d_lua::vm::closure::context::ContextMut;
use bp3d_lua::vm::RootVm;
use bp3d_lua::vm::value::function::LuaFunction;

struct TestContext {
    value3: Vec<u64>
}

decl_closure! {
    fn context_push |ctx: ContextMut<TestContext>| (val: u64) -> () {
        let mut ctx = ctx;
        ctx.value3.push(val);
    }
}

decl_closure! {
    fn context_pop |ctx: ContextMut<TestContext>| () -> Option<u64> {
        let mut ctx = ctx;
        ctx.value3.pop()
    }
}

pub fn test_context_mlua() -> Duration {
    let lua = Lua::new();
    lua.register_userdata_type::<TestContext>(|reg| {
        reg.add_method_mut("push", |_, this, val: u64| {
            this.value3.push(val);
            Ok(())
        });
        reg.add_method_mut("pop", |_, this, _: ()| {
            Ok(this.value3.pop())
        });
    }).unwrap();
    lua.load("
        function part1(ctx)
            assert(ctx:pop() == nil)
            ctx:push(1)
            ctx:push(2)
            ctx:push(3)
        end
        function part2(ctx)
            assert(ctx:pop() == 3)
            assert(ctx:pop() == 2)
            assert(ctx:pop() == 1)
            assert(ctx:pop() == nil)
            assert(ctx:pop() == nil)
        end
    ").eval::<()>().unwrap();
    let part1: Function = lua.globals().get("part1").unwrap();
    let part2: Function = lua.globals().get("part2").unwrap();
    let mut ctx = TestContext {
        value3: Vec::new()
    };
    let time = std::time::Instant::now();
    for _ in 0..20000 {
        lua.scope(|l| {
            let ud = l.create_any_userdata_ref_mut(&mut ctx).unwrap();
            part1.call::<()>(ud).unwrap();
            Ok(())
        }).unwrap();
        lua.scope(|l| {
            let ud = l.create_any_userdata_ref_mut(&mut ctx).unwrap();
            part2.call::<()>(ud).unwrap();
            Ok(())
        }).unwrap();
    }
    let time = time.elapsed();
    time
}

pub fn test_context_vm() -> Duration {
    let vm = RootVm::new();
    let ctx = ContextMut::new(&vm);
    vm.set_global(c"context_push", context_push(ctx)).unwrap();
    vm.set_global(c"context_pop", context_pop(ctx)).unwrap();
    vm.run_code::<()>(c"
        function part1()
            assert(context_pop() == nil)
            context_push(1)
            context_push(2)
            context_push(3)
        end
        function part2()
            assert(context_pop() == 3)
            assert(context_pop() == 2)
            assert(context_pop() == 1)
            assert(context_pop() == nil)
            assert(context_pop() == nil)
        end
    ").unwrap();
    let part1: LuaFunction = vm.get_global("part1").unwrap();
    let part2: LuaFunction = vm.get_global("part2").unwrap();
    let mut obj = TestContext {
        value3: vec![],
    };
    let time = std::time::Instant::now();
    for _ in 0..20000 {
        {
            let _obj = ctx.bind(&vm, &mut obj);
            part1.call::<()>(()).unwrap();
        }
        {
            let _obj = ctx.bind(&vm, &mut obj);
            part2.call::<()>(()).unwrap();
        }
    }
    let time = time.elapsed();
    time
}

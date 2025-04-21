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

use bp3d_lua::vm::table::Table;
use bp3d_lua::vm::RootVm;

#[test]
fn tables() {
    let mut vm = RootVm::new();
    let top = vm.top();
    vm.scope(|vm| {
        let mut tbl = Table::new(vm);
        tbl.set_field(c"a", 0.42)?;
        tbl.set_field(c"b", "My great string")?;
        let mut new_table = Table::new(vm);
        new_table.set_field(c"whatever", 42)?;
        let s: &str = tbl.get_field(c"b")?;
        assert_eq!(s, "My great string");
        tbl.set_field(c"sub", new_table)?;
        assert_eq!(tbl.len(), 3);
        vm.set_global(c"myTable", tbl)
    })
    .unwrap();
    let new_top = vm.top();
    assert_eq!(top, new_top);
    let v = vm.run_code::<f64>(c"return myTable.c");
    assert!(v.is_err());
    let v = vm.run_code::<f64>(c"return myTable.a");
    assert!(v.is_ok());
    assert_eq!(v.unwrap(), 0.42);
    let v = vm.run_code::<&str>(c"return myTable.b");
    assert!(v.is_ok());
    assert_eq!(v.unwrap(), "My great string");
    let v = vm.run_code::<i64>(c"return myTable.sub.whatever");
    assert!(v.is_ok());
    assert_eq!(v.unwrap(), 42);
    vm.clear();
    let new_top_1 = vm.top();
    assert_eq!(new_top, new_top_1);
    vm.scope(|vm| {
        let tbl: Table = vm.get_global("myTable")?;
        assert_eq!(tbl.len(), 3);
        let v: f64 = tbl.get_field(c"a")?;
        assert_eq!(v, 0.42);
        let v = vm.run_code::<&str>(c"return myTable.b")?;
        assert_eq!(v, "My great string");
        {
            let v: f64 = tbl.get_field(c"a")?;
            assert_eq!(v, 0.42);
        }
        assert_eq!(v, "My great string");
        Ok(())
    })
    .unwrap();
    assert_eq!(vm.top(), new_top);
}

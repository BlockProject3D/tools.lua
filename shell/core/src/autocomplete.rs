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

use std::collections::HashSet;
use bp3d_lua::decl_closure;
use bp3d_lua::libs::Lib;
use bp3d_lua::util::Namespace;
use bp3d_lua::vm::closure::rc::Rc;
use bp3d_lua::vm::table::Table;
use bp3d_lua::vm::value::any::AnyValue;
use crate::data::DataOut;

pub enum Mode {
    AddUpdate(Vec<Completions>),
    Delete(Vec<String>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    Function,
    Attribute
}

pub struct Item {
    pub name: String,
    pub ty: Type
}

impl Item {
    pub fn from_lua(name: &str, val: &AnyValue) -> Self {
        match val {
            AnyValue::Function(_) => Item { name: name.into(), ty: Type::Function },
            _ => Item { name: name.into(), ty: Type::Attribute }
        }
    }
}

pub struct Completions {
    pub path: String,
    pub items: Vec<Item>
}

fn get_capacity(val: &AnyValue) -> usize {
    match val {
        AnyValue::Function(_) => 0,
        AnyValue::Table(v) => v.len(),
        AnyValue::UserData(_) => 1,
        _ => 0
    }
}

fn list_table_completions(set: &mut HashSet<usize>, path: Vec<String>, root: &mut Vec<Completions>, mut value: Table, metatables: bool) -> bp3d_lua::vm::Result<()> {
    if set.contains(&value.uid()) {
        return Ok(());
    }
    for res in value.iter() {
        let (k, v) = res?;
        match k {
            AnyValue::String(name) => {
                let c = get_capacity(&v);
                if c > 0 {
                    let mut path = path.clone();
                    path.push(name.into());
                    root.push(Completions {
                        path: path.join("."),
                        items: Vec::with_capacity(c)
                    });
                    list_completions(set, path, root, v, metatables)?;
                } else {
                    root.last_mut().unwrap().items.push(Item::from_lua(name, &v));
                }
            }
            _ => continue
        }
    }
    if metatables {
        if let Some(tbl) = value.get_metatable() {
            list_table_completions(set, path, root, tbl, metatables)?;
        }
    }
    set.insert(value.uid());
    Ok(())
}

fn list_completions(set: &mut HashSet<usize>, path: Vec<String>, root: &mut Vec<Completions>, value: AnyValue, metatables: bool) -> bp3d_lua::vm::Result<()> {
    match value {
        AnyValue::Table(v) => list_table_completions(set, path, root, v, metatables),
        AnyValue::UserData(v) => {
            if let Some(tbl) = v.get_metatable() {
                // We assume userdata have a single metatable (following current bp3d-lua pattern).
                list_table_completions(set, path, root, tbl, false)?;
            }
            Ok(())
        }
        _ => Ok(())
    }
}

decl_closure! {
    fn build_completions |ch: Rc<DataOut>| (lua: &Vm, name: &str, metatables: bool) -> bp3d_lua::vm::Result<()> {
        let value: AnyValue = lua.get_global(name)?;
        let mut root = Vec::new();
        let mut set = HashSet::new();
        list_completions(&mut set, vec![name.into()], &mut root, value, metatables)?;
        ch.send(crate::data_out::Autocomplete(Mode::AddUpdate(root)));
        Ok(())
    }
}

decl_closure! {
    fn delete_completions |ch: Rc<DataOut>| (lua: &Vm, name: &str, metatables: bool) -> bp3d_lua::vm::Result<()> {
        let value: AnyValue = lua.get_global(name)?;
        let mut root = Vec::new();
        let mut set = HashSet::new();
        list_completions(&mut set, vec![name.into()], &mut root, value, metatables)?;
        let base = root.into_iter().map(|v| v.path);
        ch.send(crate::data_out::Autocomplete(Mode::Delete(base.collect())));
        Ok(())
    }
}

pub struct Autocomplete(std::rc::Rc<DataOut>);

impl Autocomplete {
    pub fn new(logger: DataOut) -> Autocomplete {
        Autocomplete(logger.into())
    }
}

impl Lib for Autocomplete {
    const NAMESPACE: &'static str = "bp3d.lua.shell";

    fn load(&self, namespace: &mut Namespace) -> bp3d_lua::vm::Result<()> {
        let rc = Rc::from_rust(namespace.vm(), self.0.clone());
        let rc1 = Rc::from_rust(namespace.vm(), self.0.clone());
        namespace.add([
            ("buildCompletions", build_completions(rc)),
            ("deleteCompletions", delete_completions(rc1))
        ])
    }
}

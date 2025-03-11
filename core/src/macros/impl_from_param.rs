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

#[macro_export]
macro_rules! _impl_from_param_struct {
    (
        $name:ident <$life: lifetime> {
            $($value_name: ident: $value_ty: ty),*
        }
    ) => {
        impl<$life> $crate::vm::function::FromParam<$life> for $name<$life> {
            unsafe fn from_param(vm: &'a $crate::vm::Vm, index: i32) -> Self {
                unsafe { $crate::ffi::laux::luaL_checktype(vm.as_ptr(), index, $crate::ffi::lua::Type::Table); }
                let mut top = vm.top();
                $(
                    unsafe { $crate::ffi::lua::lua_getfield(vm.as_ptr(), index, $crate::c_stringify!($value_name).as_ptr()); }
                    top += 1;
                    let $value_name = unsafe { <$value_ty as $crate::vm::function::FromParam>::from_param(vm, top) };
                )*
                Self { $($value_name),* }
            }
        }
    };
    (
        $name:ident {
            $($value_name: ident: $value_ty: ty),*
        }
    ) => {
        impl<'a> $crate::vm::function::FromParam<'a> for $name {
            unsafe fn from_param(vm: &'a $crate::vm::Vm, index: i32) -> Self {
                unsafe { $crate::ffi::laux::luaL_checktype(vm.as_ptr(), index, $crate::ffi::lua::Type::Table); }
                let mut top = vm.top();
                $(
                    unsafe { $crate::ffi::lua::lua_getfield(vm.as_ptr(), index, $crate::c_stringify!($value_name).as_ptr()); }
                    top += 1;
                    let $value_name = unsafe { <$value_ty as $crate::vm::function::FromParam>::from_param(vm, top) };
                )*
                Self { $($value_name),* }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_param {
    (
        $vis: vis struct $name:ident $(<$life: lifetime>)? {
            $($value_vis: vis $value_name: ident: $value_ty: ty),*
        }
    ) => {
        $vis struct $name $(<$life>)? {
            $($value_vis $value_name: $value_ty),*
        }

        unsafe impl$(<$life>)? $crate::vm::util::SimpleDrop for $name$(<$life>)? {}

        impl$(<$life>)? $crate::vm::util::LuaType for $name$(<$life>)? {
            fn lua_type() -> Vec<$crate::vm::util::TypeName> {
                let mut ret = Vec::new();
                $(
                    for v in <$value_ty as $crate::vm::util::LuaType>::lua_type() {
                        ret.push(v);
                    }
                )*
                ret
            }
        }

        $crate::_impl_from_param_struct! {
            $name $(<$life>)? {
                $($value_name: $value_ty),*
            }
        }
    };
}

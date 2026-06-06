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
macro_rules! _impl_userdata_static {
    ($registry: ident field $field_name: ident = $field_value: expr) => {
        $registry.add_field($crate::c_stringify!($field_name), $field_value)?;
    };
    ($registry: ident fn $function_name: ident) => {
        $registry.add_static_field(
            $crate::c_stringify!($function_name),
            $crate::vm::function::types::RFunction::wrap($function_name),
        )?;
    };
}

#[macro_export]
macro_rules! _impl_userdata {
    ($obj_name: ident, $($fn_name: ident),* { $([$($static_registry_tokens: tt)*];)* }) => {
        impl $crate::vm::userdata::UserData for $obj_name {
            fn register<C: $crate::vm::userdata::NameConvert>(registry: &$crate::vm::userdata::core::Registry<Self, C>) -> std::result::Result<(), $crate::vm::userdata::Error> {
                $(
                    let f = $obj_name::$fn_name()?;
                    registry.add_method(f);
                )*
                use $crate::vm::userdata::AddGcMethod;
                (&$crate::vm::userdata::core::AddGcMethodAuto::<$obj_name>::default()).add_gc_method(registry);
                $(
                    $crate::_impl_userdata_static!(registry $($static_registry_tokens)*);
                )*
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! decl_userdata {
    (
        $(#[$meta: meta])*
        $vis: vis struct $name:ident
    ) => {
        $(#[$meta])*
        $vis struct $name;

        unsafe impl $crate::vm::userdata::UserDataType for $name {
            const CLASS_NAME: &'static std::ffi::CStr = $crate::c_stringify!($name);
            const FULL_TYPE: &'static std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(concat!(module_path!(), "::", stringify!($name), "\0").as_ptr() as _) };
        }
    };
    (
        $(#[$meta: meta])*
        $vis: vis struct $name:ident { $($struct_decl: tt)* }
    ) => {
        $(#[$meta])*
        $vis struct $name { $($struct_decl)* }

        unsafe impl $crate::vm::userdata::UserDataType for $name {
            const CLASS_NAME: &'static std::ffi::CStr = $crate::c_stringify!($name);
            const FULL_TYPE: &'static std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(concat!(module_path!(), "::", stringify!($name), "\0").as_ptr() as _) };
        }
    };
    (
        $(#[$meta: meta])*
        $vis: vis struct $name:ident($($struct_decl: tt)*)
    ) => {
        $(#[$meta])*
        $vis struct $name($($struct_decl)*);

        unsafe impl $crate::vm::userdata::UserDataType for $name {
            const CLASS_NAME: &'static std::ffi::CStr = $crate::c_stringify!($name);
            const FULL_TYPE: &'static std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(concat!(module_path!(), "::", stringify!($name), "\0").as_ptr() as _) };
        }
    };
}

#[macro_export]
macro_rules! impl_userdata {
    (
        impl $obj_name: ident {
            $(
                $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &$obj_name2: ident $($tokens: tt)*) -> $ret_ty: ty $code: block
            )*
        }

        $(static { $([$($static_registry_tokens: tt)*];)* })?
    ) => {
        $(
            $crate::decl_userdata_func! {
                $vis fn $fn_name $(<$lifetime>)? ($this: &$obj_name $($tokens)*) -> $ret_ty $code
            }
        )*

        $crate::_impl_userdata!($obj_name, $($fn_name),* { $($([$($static_registry_tokens)*];)*)? });

        unsafe impl $crate::vm::userdata::UserDataImmutable for $obj_name {}
    };
}

#[macro_export]
macro_rules! impl_userdata_mut {
    (
        impl $obj_name: ident {
            $(
                $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($($tokens: tt)*) -> $ret_ty: ty $code: block
            )*
        }

        $(static { $([$($static_registry_tokens: tt)*];)* })?
    ) => {
        $(
            $crate::decl_userdata_func! {
                $vis fn $fn_name $(<$lifetime>)? ($($tokens)*) -> $ret_ty $code
            }
        )*

        $crate::_impl_userdata!($obj_name, $($fn_name),* { $($([$($static_registry_tokens)*];)*)? });
    };
}

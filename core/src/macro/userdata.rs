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
macro_rules! _impl_userdata {
    ($obj_name: ident, $($fn_name: ident),*) => {
        impl $crate::vm::userdata::UserData for $obj_name {
            const CLASS_NAME: &'static std::ffi::CStr = $crate::c_stringify!($obj_name);

            fn register<C: $crate::vm::userdata::NameConvert>(registry: &$crate::vm::userdata::core::Registry<Self, C>) -> Result<(), $crate::vm::userdata::Error> {
                $(
                    let (name, func) = unsafe { $obj_name::$fn_name().build()? };
                    registry.add_method(name, func);
                )*
                use $crate::vm::userdata::AddGcMethod;
                (&$crate::vm::userdata::core::AddGcMethodAuto::<$obj_name>::default()).add_gc_method(registry);
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! decl_userdata {
    (
        impl $obj_name: ident {
            $(
                $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &$obj_name2: ident $($tokens: tt)*) -> $ret_ty: ty $code: block
            )*
        }
    ) => {
        $(
            $crate::decl_userdata_func! {
                $vis fn $fn_name $(<$lifetime>)? ($this: &$obj_name $($tokens)*) -> $ret_ty $code
            }
        )*

        $crate::_impl_userdata!($obj_name, $($fn_name),*);

        unsafe impl $crate::vm::userdata::UserDataImmutable for $obj_name {}
    };
}

#[macro_export]
macro_rules! decl_userdata_mut {
    (
        impl $obj_name: ident {
            $(
                $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($($tokens: tt)*) -> $ret_ty: ty $code: block
            )*
        }
    ) => {
        $(
            $crate::decl_userdata_func! {
                $vis fn $fn_name $(<$lifetime>)? ($($tokens)*) -> $ret_ty $code
            }
        )*

        $crate::_impl_userdata!($obj_name, $($fn_name),*);
    };
}

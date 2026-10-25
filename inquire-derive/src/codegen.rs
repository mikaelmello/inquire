use proc_macro::TokenStream;
use quote::quote;
use syn::{Data::Enum, DeriveInput};

use crate::utils::enum_name_to_module_name;

pub fn generate_selectable_impl(input: DeriveInput) -> TokenStream {
    let enum_name = &input.ident;
    let variants = if let Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|v| &v.ident)
            .collect::<Vec<_>>()
    } else {
        panic!("Selectable only supports enums")
    };

    let module_name = enum_name_to_module_name(&enum_name.to_string());

    let expanded = quote! {
        mod #module_name {
            use super::*;

            #[doc(hidden)]
            pub trait Variants<T: 'static> {
                const VARIANTS: &'static [T];
            }

            impl Variants<#enum_name> for #enum_name {
                const VARIANTS: &'static [#enum_name] = &[#(#enum_name::#variants),*];
            }

            pub use Variants as VariantsTrait;
        }

        impl #enum_name {
            pub fn select(msg: &str) -> ::inquire::Select<'_, Self>
            where
                Self: ::std::fmt::Display + ::std::fmt::Debug + Copy + Clone + 'static
            {
                ::inquire::Select::new(msg, <Self as #module_name::VariantsTrait<Self>>::VARIANTS.to_vec())
            }

            pub fn multi_select(msg: &str) -> ::inquire::MultiSelect<'_, Self>
            where
                Self: ::std::fmt::Display + ::std::fmt::Debug + Copy + Clone + 'static
            {
                ::inquire::MultiSelect::new(msg, <Self as #module_name::VariantsTrait<Self>>::VARIANTS.to_vec())
            }
        }
    };

    TokenStream::from(expanded)
}

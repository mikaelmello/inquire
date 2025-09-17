use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data::Enum, DeriveInput, Ident, Lit, Meta};

use crate::utils::enum_name_to_module_name;

pub fn generate_selectable_impl(input: DeriveInput) -> TokenStream {
    let enum_name = &input.ident;
    let (variants, variant_docs): (Vec<&Ident>, Vec<Option<String>>) = {
        if let Enum(data_enum) = &input.data {
            let mut variants = Vec::new();
            let mut docs = Vec::new();

            for variant in data_enum.variants.iter() {
                let var_ident = &variant.ident;
                let doc_comment: Option<String> = get_doc_comment(&variant.attrs);

                variants.push(var_ident);
                docs.push(doc_comment);
            }

            (variants, docs)
        } else {
            panic!("Selectable only supports enums")
        }
    };

    let has_any_docs = variant_docs.iter().any(|doc| doc.is_some());
    let module_name = enum_name_to_module_name(&enum_name.to_string());
    let display_impl = if has_any_docs {
        let match_arms: Vec<proc_macro2::TokenStream> = variants
            .iter()
            .zip(variant_docs.iter())
            .map(|(variant, doc)| {
                let display_text = doc.as_ref().unwrap_or(&variant.to_string()).clone();
                quote! {
                    #enum_name::#variant => write!(f, "{}", #display_text)
                }
            })
            .collect();

        Some(quote! {
            impl ::std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
        })
    } else {
        None
    };

    let trait_bounds = if has_any_docs {
        quote! {
            where
                Self: ::std::fmt::Debug + Copy + Clone + 'static
        }
    } else {
        quote! {
            where
                Self: ::std::fmt::Display + ::std::fmt::Debug + Copy + Clone + 'static
        }
    };

    let display_check = if !has_any_docs {
        quote! {
            const _: fn() = || {
                fn assert_display<T: ::std::fmt::Display>() {}
                assert_display::<#enum_name>();
            };
        }
    } else {
        quote! {}
    };

    let variant_array: Vec<proc_macro2::TokenStream> = variants
        .iter()
        .map(|variant: &&Ident| {
            quote! { #enum_name::#variant }
        })
        .collect();

    let expanded = quote! {
        #display_impl

        mod #module_name {
            use super::*;

            #[doc(hidden)]
            pub trait Variants<T: 'static> {
                const VARIANTS: &'static [T];
            }

            impl Variants<#enum_name> for #enum_name {
                const VARIANTS: &'static [#enum_name] = &[#(#variant_array),*];
            }

            pub use Variants as VariantsTrait;
        }

        #display_check

        impl #enum_name {
            pub fn select(msg: &str) -> ::inquire::Select<'_, Self>
            #trait_bounds
            {
                let variants_vec: Vec<Self> = <Self as #module_name::VariantsTrait<Self>>::VARIANTS.to_vec();
                ::inquire::Select::new(msg, variants_vec)
            }

            pub fn multi_select(msg: &str) -> ::inquire::MultiSelect<'_, Self>
            #trait_bounds
            {
                let variants_vec: Vec<Self> = <Self as #module_name::VariantsTrait<Self>>::VARIANTS.to_vec();
                ::inquire::MultiSelect::new(msg, variants_vec)
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_doc_comment(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value().trim().to_string());
                    }
                }
            }
        }
    }

    None
}

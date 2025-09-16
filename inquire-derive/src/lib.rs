use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data::Enum, DataEnum, DeriveInput, Variant};

#[proc_macro_derive(Selectable, attributes(desc))]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let variants = match &input.data {
        Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Selectable can only be derived for enums"),
    };

    let variant_matches = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let desc = get_doc_comment(variant).unwrap_or(variant_name.to_string());

        quote! {
            #enum_name::#variant_name => #desc,
        }
    });

    let variants = if let Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|v| &v.ident)
            .collect::<Vec<_>>()
    } else {
        panic!("Selectable only supports enums")
    };

    let module_name = format_ident!("__enum_choice_for_{}", enum_name.to_string().to_lowercase());

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
            fn description(&self) -> &'static str {
                match self {
                    #(#variant_matches)*
                }
            }

            pub fn select(msg: &str) -> ::inquire::error::InquireResult<Self>
            where
                Self: ::std::fmt::Display + ::std::fmt::Debug + Copy + Clone + 'static
            {
                let answer: Self = ::inquire::Select::new(msg, <Self as #module_name::VariantsTrait<Self>>::VARIANTS.to_vec())
                    .prompt()?;

                Ok(answer)
            }
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.description())
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_doc_comment(variant: &Variant) -> Option<String> {
    let mut lines: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }),
                ..
            }) => Some(s.value()),
            _ => None,
        })
        .skip_while(|s| s.trim().is_empty())
        .flat_map(|s| {
            s.split('\n')
                .map(|s| {
                    let s = s.strip_prefix(' ').unwrap_or(s);
                    s.to_owned()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    while let Some(true) = lines.last().map(|s| s.trim().is_empty()) {
        lines.pop();
    }

    Some(lines.concat())
}

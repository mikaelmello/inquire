use darling::{ast, util, FromDeriveInput};
use quote::quote;

use crate::field::{FieldMultiContext, FieldSingleContext};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inquire), supports(struct_named))]
pub struct InquireFormOpts {
    /// Current struct ident
    pub ident: syn::Ident,
    /// Option to set inquire_#fieldname as private
    pub private: Option<bool>,
    /// All fields
    pub data: ast::Data<util::Ignored, FieldMultiContext>,
}

impl InquireFormOpts {
    pub fn gen(self) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let fields = self
            .data
            .take_struct()
            .take()
            .map(|item| {
                item.fields
                    .into_iter()
                    .filter(|field| !(matches!(field.skip, Some(true))))
                    .map(|field| FieldSingleContext {
                        ident: field.ident.clone().expect("field does not have an ident"),
                        private: self.private,
                        ty: field.ty.clone(),
                        field_type: field.parse().expect("field does not have a field_type"),
                    })
                    .collect::<Vec<_>>()
            })
            .expect("inquire-derive must be used on struct");

        // Generate Methods' impls
        let (methods, errs) =
            fields
                .iter()
                .fold((Vec::new(), Vec::new()), |(mut acc, mut acce), elem| {
                    let method = elem.generate_inquire_method();
                    match method {
                        Ok(ts) => acc.push(ts),
                        Err(e) => acce.extend(e),
                    }
                    (acc, acce)
                });

        let methods = if errs.is_empty() {
            Ok(methods)
        } else {
            // TODO: improve error handling
            Err(errs.get(0).expect("fail on error conversion").clone())
        }?;

        let methods = quote! {
            #(#methods)*
        };

        // Generate Methods' calls
        let (methods_calls, errs) =
            fields
                .iter()
                .fold((Vec::new(), Vec::new()), |(mut acc, mut acce), elem| {
                    let method = elem.generate_inquire_method_call();
                    match method {
                        Ok(ts) => acc.push(ts),
                        Err(e) => acce.extend(e),
                    }
                    (acc, acce)
                });

        let methods_calls = if errs.is_empty() {
            Ok(methods_calls)
        } else {
            // TODO: improve error handling
            Err(errs.get(0).expect("failed on error conversion").clone())
        }?;

        let inquire = quote! {
                /// Will invoke inquire prompts on mutable `Self`
                pub fn inquire_mut(&mut self) -> inquire::error::InquireResult<()> {
                    #(#methods_calls)*
                    Ok(())
                }

                /// Will invoke inquire prompts on new instance from Default
                pub fn from_inquire() -> inquire::error::InquireResult<Self> {
                    let mut s = Self::default();
                    s.inquire_mut()?;
                    Ok(s)
                }
        };

        // General implementation
        Ok(quote! {
            impl #ident {
                #methods
                #inquire
            }
        })
    }
}

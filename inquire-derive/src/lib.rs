//! # Inquire Derive
//!
//!
use crate::structural::InquireFormOpts;
use proc_macro::TokenStream;
use syn::DeriveInput;

pub(crate) mod field;
pub(crate) mod helpers;
pub(crate) mod prompts;
pub(crate) mod structural;

#[proc_macro_derive(InquireForm, attributes(inquire))]
pub fn derive_inquire(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Error InquireForm derive");
    let parsed: Result<InquireFormOpts, darling::Error> =
        darling::FromDeriveInput::from_derive_input(&ast);
    // println!("{:?}", parsed);
    parsed
        .unwrap()
        .gen()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

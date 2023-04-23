//! # Inquire Derive
//!
//!

#![warn(
    // missing_docs,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::map_err_ignore,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::pattern_type_mismatch,
    clippy::string_slice,
    clippy::try_err
)]
// clippy deny/error level lints, they always have  quick fix that should be preferred
#![deny(
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::string_add,
    clippy::string_to_string,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::verbose_file_reads
)]
// allowed rules
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::redundant_pub_crate,
    clippy::indexing_slicing
)]

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
    match parsed
        .map_err(darling::Error::write_errors)
        .map(structural::InquireFormOpts::gen)
        .map(|gen| gen.unwrap_or_else(|err| err.to_compile_error()))
    {
        Ok(result) => result.into(),
        Err(err) => err.into(),
    }
}

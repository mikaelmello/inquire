use proc_macro2::TokenStream;

use crate::field::FieldSingleContext;

/// [`FieldInvokeInquire`] interfacing field_type impl
pub trait FieldInquireForm {
    /// Generate the call for inquire_#fieldname
    fn generate_inquire_method_call(
        &self,
        _ctx: &FieldSingleContext,
    ) -> Result<TokenStream, Vec<syn::Error>>;

    /// Generate the `fn` inquire_#fieldname
    fn generate_inquire_method(
        &self,
        _ctx: &FieldSingleContext,
    ) -> Result<TokenStream, Vec<syn::Error>>;
}

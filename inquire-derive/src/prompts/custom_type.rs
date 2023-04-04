use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Expr;

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

/// CustomType prompts are generic prompts suitable for when you need to parse the user input into a specific type, for example an f64 or a rust_decimal, maybe even an uuid.
#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct CustomType {
    /// Message to be presented to the user.
    pub prompt_message: Option<Expr>,
    /// Default value returned when the user submits an empty response.
    pub default_value: Option<Expr>,
    /// Short hint that describes the expected value of the input.
    pub placeholder_value: Option<Expr>,
    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: Option<Expr>,
    /// Function that formats how the default value is displayed to the user.
    /// By default, displays "y/n" with the default value capitalized, e.g. "y/N".
    pub default_value_formatter: Option<Expr>,
    /// Help message to be presented to the user.
    pub help_message: Option<Expr>,
    /// Function that parses the user input and returns the result value.
    pub parser: Option<Expr>,
    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Option<Expr>,
    /// Error message to display when a value could not be parsed from the input.
    pub error_message: Option<Expr>,
}

impl FieldInquireForm for CustomType {
    fn generate_inquire_method(
        &self,
        ctx: &FieldSingleContext,
    ) -> Result<TokenStream, Vec<syn::Error>> {
        // contextual parameters
        let fieldname = ctx.ident.to_string();
        let fieldname_idt = format_ident!("{}", fieldname);
        let method_name = format_ident!("inquire_{}", fieldname);
        let ty = &ctx.ty;
        let visibility = match ctx.private {
            Some(value) if value => {
                quote! {}
            }
            _ => {
                quote! { pub }
            }
        };

        // generate ident
        let prompt_message = match &self.prompt_message {
            Some(prompt_message) => prompt_message.to_token_stream(),
            None => {
                let prompt_message = format!("What's your {}?", fieldname);
                quote! {
                    #prompt_message
                }
            }
        };
        let default_value = match &self.default_value {
            Some(default_value) => quote! { Some(#default_value) },
            None => quote! { Some(self.#fieldname_idt.clone()) },
        };
        let placeholder_value = match &self.placeholder_value {
            Some(placeholder_value) => quote! { Some(#placeholder_value) },
            None => quote! { None },
        };
        let help_message = match &self.help_message {
            Some(help_message) => quote! { Some(#help_message) },
            None => quote! { None },
        };
        let formatter = match &self.formatter {
            Some(formatter) => quote! { #formatter },
            None => quote! { &|val| val.to_string() },
        };
        let default_value_formatter = match &self.default_value_formatter {
            Some(default_value_formatter) => quote! { #default_value_formatter },
            None => quote! { &|val| val.to_string() },
        };
        let parser = match &self.parser {
            Some(parser) => quote! { #parser },
            None => quote! { &|a| a.parse::<#ty>().map_err(|_| ())},
        };
        let validators = match &self.validators {
            Some(validators) => quote! { #validators },
            None => quote! { inquire::CustomType::DEFAULT_VALIDATORS },
        };
        let error_message = match &self.error_message {
            Some(error_message) => quote! { String::from(#error_message) },
            None => quote! { "Invalid input".to_string() },
        };

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<#ty> {
                inquire::CustomType::<#ty> {
                    message: #prompt_message,
                    default: #default_value,
                    help_message: #help_message,
                    formatter: #formatter,
                    placeholder: #placeholder_value,
                    parser: #parser,
                    default_value_formatter: #default_value_formatter,
                    error_message: #error_message,
                    validators: #validators,
                    render_config: inquire::ui::RenderConfig::default(),
                }
                .prompt()
            }
        })
    }

    fn generate_inquire_method_call(
        &self,
        ctx: &FieldSingleContext,
    ) -> Result<TokenStream, Vec<syn::Error>> {
        let fieldname = format_ident!("{}", ctx.ident.to_string());
        let method_name = format_ident!("inquire_{}", ctx.ident.to_string());
        Ok(quote! {
           self.#fieldname = self.#method_name()?;
        })
    }
}

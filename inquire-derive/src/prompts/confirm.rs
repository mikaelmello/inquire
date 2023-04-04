use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Expr;

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

/// Confirm is a prompt to ask the user for simple yes/no questions, commonly known by asking the user displaying the (y/n) text.
#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct Confirm {
    /// Required when creating the prompt.
    pub prompt_message: Option<Expr>,
    /// Default value returned when the user submits an empty response.
    pub default_value: Option<Expr>,
    /// Short hint that describes the expected value of the input.
    pub placeholder_value: Option<Expr>,
    /// Message displayed at the line below the prompt.
    pub help_message: Option<Expr>,
    /// Custom formatter in case you need to pre-process the user input before showing it as the final answer.
    /// Formats true to "Yes" and false to "No", by default.
    pub formatter: Option<Expr>,
    /// Custom parser for user inputs.
    /// The default bool parser returns true if the input is either "y" or "yes", in a case-insensitive comparison. Similarly, the parser returns false if the input is either "n" or "no".
    pub parser: Option<Expr>,
    /// Function that formats how the default value is displayed to the user.
    /// By default, displays "y/n" with the default value capitalized, e.g. "y/N".
    pub default_value_formatter: Option<Expr>,
    /// Error message to display when a value could not be parsed from the input.
    /// Set to "Invalid answer, try typing 'y' for yes or 'n' for no" by default.
    pub error_message: Option<Expr>,
}

impl FieldInquireForm for Confirm {
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
            None => quote! { Some(self.#fieldname_idt) },
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
            None => quote! { inquire::Confirm::DEFAULT_FORMATTER },
        };
        let parser = match &self.parser {
            Some(parser) => quote! { #parser },
            None => quote! { inquire::Confirm::DEFAULT_PARSER },
        };
        let default_value_formatter = match &self.default_value_formatter {
            Some(default_value_formatter) => quote! { #default_value_formatter },
            None => quote! { inquire::Confirm::DEFAULT_DEFAULT_VALUE_FORMATTER },
        };
        let error_message = match &self.error_message {
            Some(error_message) => quote! { String::from(#error_message) },
            None => quote! { String::from(inquire::Confirm::DEFAULT_ERROR_MESSAGE) },
        };

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<#ty> {
                inquire::Confirm {
                    message: #prompt_message,
                    default: #default_value,
                    placeholder: #placeholder_value,
                    help_message: #help_message,
                    formatter: #formatter,
                    parser: #parser,
                    default_value_formatter: #default_value_formatter,
                    error_message: #error_message,
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

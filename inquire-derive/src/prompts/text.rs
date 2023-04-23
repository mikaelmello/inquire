#![allow(clippy::nursery, clippy::option_if_let_else)]

use darling::{FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct Text {
    /// Main message when prompting the user for input, "What is your #fieldname?" in the example above.
    #[darling(default)]
    pub prompt_message: Option<Expr>,
    /// Message displayed at the line below the prompt.
    pub help_message: Option<Expr>,
    /// Default value returned when the user submits an empty response.
    pub default_value: Option<Expr>,
    /// Initial value of the prompt's text input, in case you want to display the prompt with something already filled in.
    pub initial_value: Option<Expr>,
    /// Short hint that describes the expected value of the input.
    pub placeholder_value: Option<Expr>,
    /// Custom validators to the user's input, displaying an error message if the input does not pass the requirements.
    pub validators: Option<Expr>,
    /// Custom formatter in case you need to pre-process the user input before showing it as the final answer.
    pub formatter: Option<Expr>,
    /// Sets a new autocompleter
    pub autocompleter: Option<Expr>,
    /// Page size of the suggestions displayed to the user, when applicable.
    pub page_size: Option<Expr>,
}

impl FieldInquireForm for Text {
    fn generate_inquire_method(&self, ctx: &FieldSingleContext) -> Result<TokenStream, Vec<Error>> {
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
        let prompt_message = self.prompt_message.as_ref().map_or_else(
            || {
                let prompt_message = format!("What's your {fieldname}?");
                quote! {
                    #prompt_message
                }
            },
            quote::ToTokens::to_token_stream,
        );

        let help_message = self.help_message.as_ref().map_or_else(
            || quote! { None },
            |help_message| quote! { Some(#help_message) },
        );

        let default_value = self.default_value.as_ref().map_or_else(
            || quote! { Some(self.#fieldname_idt.as_str()) },
            |default_value| quote! { Some(#default_value) },
        );

        let initial_value = self.initial_value.as_ref().map_or_else(
            || quote! { None },
            |initial_value| quote! { Some(#initial_value) },
        );

        let placeholder_value = self.placeholder_value.as_ref().map_or_else(
            || quote! { None },
            |placeholder_value| quote! { Some(#placeholder_value) },
        );

        let validators = self.validators.as_ref().map_or_else(
            || quote! { Vec::new() },
            |validators| quote! { #validators },
        );

        let formatter = self.formatter.as_ref().map_or_else(
            || quote! { inquire::Text::DEFAULT_FORMATTER },
            |formatter| quote! { Some(#formatter) },
        );

        let autocompleter = self.autocompleter.as_ref().map_or_else(
            || quote! { None },
            |autocompleter| quote! { Some(Box::new(#autocompleter)) },
        );

        let page_size = self.page_size.as_ref().map_or_else(
            || quote! { inquire::Text::DEFAULT_PAGE_SIZE },
            |page_size| quote! { #page_size },
        );

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<#ty> {
                inquire::Text {
                    message: #prompt_message,
                    initial_value: #initial_value,
                    default: #default_value,
                    placeholder: #placeholder_value,
                    help_message: #help_message,
                    formatter: #formatter,
                    validators: #validators,
                    page_size: #page_size,
                    autocompleter: #autocompleter,
                    render_config: inquire::ui::RenderConfig::default(),
                }
                .prompt()
            }
        })
    }

    fn generate_inquire_method_call(
        &self,
        ctx: &FieldSingleContext,
    ) -> Result<TokenStream, Vec<Error>> {
        let fieldname = format_ident!("{}", ctx.ident.to_string());
        let method_name = format_ident!("inquire_{}", ctx.ident.to_string());
        Ok(quote! {
           self.#fieldname = self.#method_name()?;
        })
    }
}

use darling::{FromMeta, ToTokens};
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
        let prompt_message = match &self.prompt_message {
            Some(prompt_message) => prompt_message.to_token_stream(),
            None => {
                let prompt_message = format!("What's your {}?", fieldname);
                quote! {
                    #prompt_message
                }
            }
        };
        let help_message = match &self.help_message {
            Some(help_message) => quote! { Some(#help_message) },
            None => quote! { None },
        };
        let default_value = match &self.default_value {
            Some(default_value) => quote! { Some(#default_value) },
            None => quote! { Some(self.#fieldname_idt.as_str()) },
        };
        let initial_value = match &self.initial_value {
            Some(initial_value) => quote! { Some(#initial_value) },
            None => quote! { None },
        };
        let placeholder_value = match &self.placeholder_value {
            Some(placeholder_value) => quote! { Some(#placeholder_value) },
            None => quote! { None },
        };
        let validators = match &self.validators {
            Some(validators) => quote! { #validators },
            None => quote! { Vec::new() },
        };
        let formatter = match &self.formatter {
            Some(formatter) => quote! { Some(#formatter) },
            None => quote! { inquire::Text::DEFAULT_FORMATTER },
        };
        let autocompleter = match &self.autocompleter {
            Some(autocompleter) => quote! { Some(Box::new(#autocompleter)) },
            None => quote! { None },
        };
        let page_size = match &self.page_size {
            Some(page_size) => quote! { #page_size },
            None => quote! { inquire::Text::DEFAULT_PAGE_SIZE },
        };

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

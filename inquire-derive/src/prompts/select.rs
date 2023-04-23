use darling::{FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

/// Select prompts are suitable for when you need the user to select one option among many.
#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct Select {
    /// Required when creating the prompt.
    pub prompt_message: Option<Expr>,
    /// Options displayed to the user. must be non-empty.
    pub options: Option<Expr>,
    /// Index of the cursor when the prompt is first rendered. default is 0 (first option). if the index is out-of-range of the option list, the prompt will fail with an inquireerror::invalidconfiguration error.
    pub starting_cursor: Option<Expr>,
    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: Option<Expr>,
    /// Message displayed at the line below the prompt.
    pub help_message: Option<Expr>,
    /// Custom formatter in case you need to pre-process the user input before showing it as the final answer.
    /// Prints the selected option string value by default.
    pub formatter: Option<Expr>,
    /// Number of options displayed at once, 7 by default.
    pub page_size: Option<Expr>,
    /// Function that defines if an option is displayed or not based on the current filter input.
    pub filter_function: Option<Expr>,
}

impl FieldInquireForm for Select {
    fn generate_inquire_method(&self, ctx: &FieldSingleContext) -> Result<TokenStream, Vec<Error>> {
        // contextual parameters
        let fieldname = ctx.ident.to_string();
        let _fieldname_idt = format_ident!("{}", fieldname);
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

        let options = self.options.as_ref().map_or_else(
            || quote! { Vec::new() },
            quote::ToTokens::to_token_stream,
        );

        let vim_mode = self.vim_mode.as_ref().map_or_else(
            || quote! { inquire::Select::<#ty>::DEFAULT_VIM_MODE },
            |vim_mode| quote! { #vim_mode },
        );

        let filter = self.filter_function.as_ref().map_or_else(
            || quote! { inquire::Select::<#ty>::DEFAULT_FILTER },
            |filter| quote! { #filter },
        );

        let formatter = self.formatter.as_ref().map_or_else(
            || quote! { inquire::Select::<#ty>::DEFAULT_FORMATTER },
            |formatter| quote! { Some(#formatter) },
        );

        let starting_cursor = self.starting_cursor.as_ref().map_or_else(
            || quote! { inquire::Select::<#ty>::DEFAULT_STARTING_CURSOR },
            |starting_cursor| quote! { #starting_cursor },
        );

        let page_size = self.page_size.as_ref().map_or_else(
            || quote! { inquire::Select::<#ty>::DEFAULT_PAGE_SIZE },
            |page_size| quote! { #page_size },
        );

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<#ty> {
                inquire::Select::<#ty> {
                    message: #prompt_message,
                    options: #options,
                    help_message: #help_message,
                    formatter: #formatter,
                    page_size: #page_size,
                    starting_cursor: #starting_cursor,
                    filter: #filter,
                    vim_mode: #vim_mode,
                    render_config: inquire::ui::RenderConfig::default(),
                }.prompt()
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

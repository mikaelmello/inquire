use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use super::FieldInquireForm;
use crate::field::FieldSingleContext;
use crate::helpers::extract_type_from_vec;

/// MultiSelect prompts are suitable for when you need the user to select many options (including none if applicable) among a list of them.
/// The user can select (or deselect) the current highlighted option by pressing space, clean all selections by pressing the left arrow and select all options by pressing the right arrow.
#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct MultiSelect {
    /// Message to be presented to the user.
    pub prompt_message: Option<Expr>,
    /// Options displayed to the user.
    pub options: Option<Expr>,
    /// Default indexes of options to be selected from the start.
    pub default_value: Option<Expr>,
    /// Help message to be presented to the user.
    pub help_message: Option<Expr>,
    /// Page size of the options displayed to the user.
    pub page_size: Option<Expr>,
    /// Whether vim mode is enabled. When enabled, the user can
    /// navigate through the options using hjkl.
    pub vim_mode: Option<Expr>,
    /// Starting cursor index of the selection.
    pub starting_cursor: Option<Expr>,
    /// Function called with the current user input to filter the provided
    /// options.
    pub filter_function: Option<Expr>,
    /// Whether the current filter typed by the user is kept or cleaned after a selection is made.
    pub keep_filter: Option<Expr>,
    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: Option<Expr>,
    /// Validator to apply to the user input.
    /// In case of error, the message is displayed one line above the prompt.
    pub validator: Option<Expr>,
}

impl FieldInquireForm for MultiSelect {
    fn generate_inquire_method(&self, ctx: &FieldSingleContext) -> Result<TokenStream, Vec<Error>> {
        // contextual parameters
        let fieldname = ctx.ident.to_string();
        let _fieldname_idt = format_ident!("{}", fieldname);
        let method_name = format_ident!("inquire_{}", fieldname);
        let ty = &ctx.ty;
        let inner_ty = extract_type_from_vec(ty).map_err(|value| vec![value])?;
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
            Some(default_value) => quote! { #default_value },
            None => quote! {  None },
        };
        let help_message = match &self.help_message {
            Some(help_message) => quote! { Some(#help_message) },
            None => quote! { None },
        };
        let validator = match &self.validator {
            Some(validator) => quote! { Some(#validator) },
            None => quote! { None },
        };
        let options = match &self.options {
            Some(options) => options.to_token_stream(),
            None => quote! { Vec::new() },
        };
        let vim_mode = match &self.vim_mode {
            Some(vim_mode) => quote! { #vim_mode },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_VIM_MODE },
        };
        let keep_filter = match &self.keep_filter {
            Some(keep_filter) => quote! { #keep_filter },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_KEEP_FILTER },
        };
        let filter_function = match &self.filter_function {
            Some(filter_function) => quote! { #filter_function },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_FILTER },
        };
        let formatter = match &self.formatter {
            Some(formatter) => quote! { Some(#formatter) },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_FORMATTER },
        };
        let starting_cursor = match &self.starting_cursor {
            Some(starting_cursor) => quote! { #starting_cursor },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_STARTING_CURSOR },
        };
        let page_size = match &self.page_size {
            Some(page_size) => quote! { #page_size },
            None => quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_PAGE_SIZE },
        };

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<Vec<#inner_ty>> {
                inquire::MultiSelect::<#inner_ty> {
                    message: #prompt_message,
                    options: #options,
                    default: #default_value,
                    help_message: #help_message,
                    formatter: #formatter,
                    page_size: #page_size,
                    starting_cursor: #starting_cursor,
                    keep_filter: #keep_filter,
                    filter: #filter_function,
                    vim_mode: #vim_mode,
                    validator: #validator,
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

use darling::{FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use super::FieldInquireForm;
use crate::field::FieldSingleContext;
use crate::helpers::extract_type_from_vec;

/// `MultiSelect` prompts are suitable for when you need the user to select many options (including none if applicable) among a list of them.
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
        let prompt_message = self.prompt_message.as_ref().map_or_else(
            || {
                let prompt_message = format!("What's your {fieldname}?");
                quote! {
                    #prompt_message
                }
            },
            quote::ToTokens::to_token_stream,
        );
        let default_value = self.default_value.as_ref().map_or_else(
            || quote! {  None },
            |default_value| quote! { #default_value },
        );
        let help_message = self.help_message.as_ref().map_or_else(
            || quote! { None },
            |help_message| quote! { Some(#help_message) },
        );
        let validator = self
            .validator
            .as_ref()
            .map_or_else(|| quote! { None }, |validator| quote! { Some(#validator) });
        let options = self.options.as_ref().map_or_else(
            || quote! { Vec::new() },
            quote::ToTokens::to_token_stream,
        );
        let vim_mode = self.vim_mode.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_VIM_MODE },
            |vim_mode| quote! { #vim_mode },
        );
        let keep_filter = self.keep_filter.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_KEEP_FILTER },
            |keep_filter| quote! { #keep_filter },
        );
        let filter_function = self.filter_function.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_FILTER },
            |filter_function| quote! { #filter_function },
        );
        let formatter = self.formatter.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_FORMATTER },
            |formatter| quote! { Some(#formatter) },
        );
        let starting_cursor = self.starting_cursor.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_STARTING_CURSOR },
            |starting_cursor| quote! { #starting_cursor },
        );
        let page_size = self.page_size.as_ref().map_or_else(
            || quote! { inquire::MultiSelect::<#inner_ty>::DEFAULT_PAGE_SIZE },
            |page_size| quote! { #page_size },
        );

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

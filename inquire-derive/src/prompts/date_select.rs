use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct DateSelect {
    /// Main message when prompting the user for input, "What is your name?" in the example above.
    pub prompt_message: Option<Expr>,
    /// Message displayed at the line below the prompt.
    pub help_message: Option<Expr>,
    ///  Default value selected when the calendar is displayed and the one select if the user submits without any previous actions. Current date by default.
    pub default_value: Option<Expr>,
    ///  Default value selected when the calendar is displayed and the one select if the user submits without any previous actions. Current date by default.
    pub vim_mode: Option<Expr>,
    /// Custom validators to the user's selected date, displaying an error message if the date does not pass the requirements.
    pub validators: Option<Expr>,
    /// Custom formatter in case you need to pre-process the user input before showing it as the final answer.
    /// Formats to "Month Day, Year" by default.
    pub formatter: Option<Expr>,
    /// Which day of the week should be displayed in the first column of the calendar, Sunday by default.
    pub week_start: Option<Expr>,
    /// Inclusive boundaries of allowed dates in the interactive calendar. If any boundary is set, the user will not be able to move past them, consequently not being able to select any dates out of the allowed range.
    pub min_date: Option<Expr>,
    /// Inclusive boundaries of allowed dates in the interactive calendar. If any boundary is set, the user will not be able to move past them, consequently not being able to select any dates out of the allowed range.
    pub max_date: Option<Expr>,
}

impl FieldInquireForm for DateSelect {
    fn generate_inquire_method(&self, ctx: &FieldSingleContext) -> Result<TokenStream, Vec<Error>> {
        // contextual parameters
        let fieldname = ctx.ident.to_string();
        let fieldname_idt = format_ident!("{}", fieldname);
        let method_name = format_ident!("inquire_{}", fieldname);
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
            None => quote! {  self.#fieldname_idt },
        };
        let validators = match &self.validators {
            Some(validators) => quote! { #validators },
            None => quote! { Vec::new() },
        };
        let help_message = match &self.help_message {
            Some(help_message) => quote! { Some(#help_message) },
            None => quote! { None },
        };
        let formatter = match &self.formatter {
            Some(formatter) => quote! { Some(#formatter) },
            None => quote! { inquire::DateSelect::DEFAULT_FORMATTER },
        };
        let week_start = match &self.week_start {
            Some(week_start) => quote! { #week_start },
            None => quote! { inquire::DateSelect::DEFAULT_WEEK_START },
        };
        let min_date = match &self.min_date {
            Some(min_date) => quote! { #min_date },
            None => quote! { inquire::DateSelect::DEFAULT_MIN_DATE },
        };
        let max_date = match &self.max_date {
            Some(max_date) => quote! { #max_date },
            None => quote! { inquire::DateSelect::DEFAULT_MAX_DATE },
        };
        let vim_mode = match &self.vim_mode {
            Some(vim_mode) => quote! { #vim_mode },
            None => quote! { inquire::DateSelect::DEFAULT_VIM_MODE },
        };

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<NaiveDate> {
                inquire::DateSelect {
                    message: #prompt_message,
                    help_message: #help_message,
                    starting_date: #default_value,
                    formatter: #formatter,
                    vim_mode: #vim_mode,
                    week_start: #week_start,
                    validators: #validators,
                    min_date: #min_date,
                    max_date: #max_date,
                    render_config: inquire::ui::RenderConfig::default(),
                }.prompt()
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

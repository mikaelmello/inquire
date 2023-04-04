use darling::{FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Expr;

use crate::field::FieldSingleContext;

use super::FieldInquireForm;

/// Editor prompts are meant for cases where you need the user to write some text that might not fit in a single line, such as long descriptions or commit messages.
#[derive(Debug, FromMeta, Default)]
#[darling(default)]
pub struct Editor {
    /// Message to be presented to the user.
    pub prompt_message: Option<Expr>,

    /// Command to open the editor.
    pub editor_command: Option<Expr>,

    /// Args to pass to the editor.
    pub editor_command_args: Option<Expr>,

    /// Extension of the file opened in the text editor, useful for syntax highlighting.
    ///
    /// The dot prefix should be included in the string, e.g. ".rs".
    pub file_extension: Option<Expr>,

    /// Predefined text to be present on the text file on the text editor.
    pub predefined_text: Option<Expr>,

    /// Help message to be presented to the user.
    pub help_message: Option<Expr>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: Option<Expr>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Option<Expr>,
}

impl FieldInquireForm for Editor {
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

        let editor_command_args = match &self.editor_command_args {
            Some(editor_command_args) => editor_command_args.to_token_stream(),
            None => quote! { &[] },
        };
        let editor_command = match &self.editor_command {
            Some(editor_command) => editor_command.to_token_stream(),
            // TODO: ask inquire lib to expose pub get_default_editor_command()
            // or impl our own function
            None => quote! {
                &std::ffi::OsString::from(if cfg!(windows) {
                    String::from("notepad")
                } else {
                    String::from("nano")
                })
            },
        };
        let file_extension = match &self.file_extension {
            Some(file_extension) => file_extension.to_token_stream(),
            None => quote! { ".txt" },
        };
        let predefined_text = match &self.predefined_text {
            Some(predefined_text) => quote! { Some(#predefined_text) },
            None => quote! { Some(self.#fieldname_idt.as_str()) },
        };
        let help_message = match &self.help_message {
            Some(help_message) => quote! { Some(#help_message) },
            None => quote! { inquire::Editor::DEFAULT_HELP_MESSAGE },
        };
        let validators = match &self.validators {
            Some(validators) => quote! { #validators },
            None => quote! { inquire::Editor::DEFAULT_VALIDATORS },
        };
        let formatter = match &self.formatter {
            Some(formatter) => quote! { #formatter },
            None => quote! { inquire::Editor::DEFAULT_FORMATTER },
        };

        // Generate method
        Ok(quote! {
            /// Return inquire #fieldname or an [`InquireResult`](inquire::error::InquireResult)
            #visibility fn #method_name(&self) -> inquire::error::InquireResult<#ty> {
                inquire::Editor {
                    message: #prompt_message,
                    editor_command: #editor_command,
                    editor_command_args: #editor_command_args,
                    file_extension: #file_extension,
                    predefined_text: #predefined_text,
                    help_message: #help_message,
                    validators: #validators,
                    formatter: #formatter,
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

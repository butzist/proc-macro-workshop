use output::{
    output_build_method, output_builder_constructor, output_builder_type, output_setters,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

mod field;
mod helpers;
mod input;
mod output;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let builder = builder(&input);

    builder.into()
}

fn builder(input: &DeriveInput) -> TokenStream {
    let input = match input::parse_input(input) {
        Ok(value) => value,
        Err(errs) => return errs,
    };

    let ty = output_builder_type(&input);
    let constructor = output_builder_constructor(&input);
    let setters = output_setters(&input);
    let builder = output_build_method(&input);

    quote! {
        #ty
        #constructor
        #setters
        #builder
    }
}

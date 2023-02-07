use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use syn::{DataStruct, DeriveInput, Fields};

use crate::{field::BuilderField, helpers::CollectErrorTokensExt};

pub(crate) struct Input<'a> {
    pub ident: &'a Ident,
    pub fields: Vec<BuilderField<'a>>,
}

pub(crate) fn parse_input<'a>(input: &'a DeriveInput) -> Result<Input<'a>, TokenStream> {
    let ident = &input.ident;
    let fields = match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named { 0: ref fields },
            ..
        }) => fields,
        _ => return Err(quote_spanned! { input.ident.span() => compile_error!("only classic structs are supported"); }),
    }
    .named
    .iter()
    .map(|f| f.try_into())
    .collect_errors_to_stream()?;

    Ok(Input { ident, fields })
}

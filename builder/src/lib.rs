use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Fields};

mod field;
use field::BuilderField;

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let builder = builder(&input);

    builder.into()
}

fn builder(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let builder_ident = &format_ident!("{}Builder", ident);

    let fields: Vec<_> = match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named { 0: ref fields },
            ..
        }) => fields,
        _ => unimplemented!(),
    }
    .named
    .iter()
    .map(|f| f.into())
    .collect();

    let ty = builder_type(&fields, &builder_ident);
    let constructor = builder_constructor(&fields, ident, builder_ident);
    let setters = builder_setters(&fields, builder_ident);
    let builder = builder_method(&fields, ident, builder_ident);

    quote! {
        #ty
        #constructor
        #setters
        #builder
    }
}

fn builder_type(fields: &[BuilderField], builder_ident: &Ident) -> TokenStream {
    let optional_fields = fields.iter().map(|field| match *field {
        BuilderField::Mandatory { ident, ty } | BuilderField::Optional { ident, ty } => {
            quote! { #ident: Option<#ty> }
        }
    });

    quote! {
        pub struct #builder_ident {
            #(#optional_fields),*
        }
    }
    .into()
}

fn builder_constructor(
    fields: &[BuilderField],
    ident: &Ident,
    builder_ident: &Ident,
) -> TokenStream {
    let field_initializers = fields.iter().map(|field| match *field {
        BuilderField::Mandatory { ident, .. } | BuilderField::Optional { ident, .. } => {
            quote! { #ident: None }
        }
    });

    quote! {
        impl #ident {
             pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#field_initializers),*
                }
            }
        }
    }
    .into()
}

fn builder_setters(fields: &[BuilderField], builder_ident: &Ident) -> TokenStream {
    let field_setters = fields.iter().map(|field| match *field {
        BuilderField::Mandatory { ident, ty } | BuilderField::Optional { ident, ty } => quote! {
            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        },
    });

    quote! {
        impl #builder_ident {
            #(#field_setters)*
        }
    }
    .into()
}

fn builder_method(fields: &[BuilderField], ident: &Ident, builder_ident: &Ident) -> TokenStream {
    let build_fields = fields.iter().map(|field| match *field {
        BuilderField::Optional { ident, .. } => quote! {
            #ident: self.#ident.take()
        },
        BuilderField::Mandatory { ident, .. } => {
            let msg = format!("{} not set", ident);
            quote! {
                #ident: self.#ident.take().ok_or(#msg)?
            }
        }
    });

    quote! {
        impl #builder_ident {
            pub fn build(&mut self) -> Result<#ident, Box<dyn std::error::Error>> {
                Ok(#ident {
                    #(#build_fields),*
                })
            }
        }

    }
    .into()
}

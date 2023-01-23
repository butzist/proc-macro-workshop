use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Fields};

mod field;
use field::BuilderField;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let builder = builder(&input);
    eprintln!("{}", &builder);

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
        BuilderField::Multi { ident, ty, .. } => {
            quote! { #ident: #ty }
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
            quote! { #ident: Option::None }
        }
        BuilderField::Multi { ident, .. } => {
            quote! { #ident: ::core::default::Default::default() }
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
    let field_setters = fields.iter().flat_map(|field| match *field {
        BuilderField::Mandatory { ident, ty } | BuilderField::Optional { ident, ty } => {
            Some(quote! {
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            })
        }
        BuilderField::Multi {
            ident,
            ty,
            with_set_all,
            ..
        } => with_set_all.then_some(quote! {
            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = #ident;
                self
            }
        }),
    });

    let multi_setters = fields.iter().flat_map(|field| match *field {
        BuilderField::Multi {
            ident,
            elem_ty,
            ref attrs,
            ..
        } => attrs
            .iter()
            .map(|attr| {
                quote! {
                    fn #attr(&mut self, #attr: #elem_ty) -> &mut Self {
                        self.#ident.push(#attr);
                        self
                    }
                }
            })
            .collect(),
        _ => vec![],
    });

    quote! {
        impl #builder_ident {
            #(#field_setters)*
            #(#multi_setters)*
        }
    }
    .into()
}

fn builder_method(fields: &[BuilderField], ident: &Ident, builder_ident: &Ident) -> TokenStream {
    let build_fields = fields.iter().map(|field| match *field {
        BuilderField::Optional { ident, .. } | BuilderField::Multi { ident, .. } => quote! {
            #ident: ::std::mem::take(&mut self.#ident)
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

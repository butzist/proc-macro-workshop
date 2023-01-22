use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, DataStruct, DeriveInput, Field, Fields, GenericArgument,
    PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let builder = builder(&input);
    builder.into()
}

fn builder(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let builder_ident = &Ident::new(&format!("{}Builder", ident), Span::call_site());

    let fields = match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named { 0: ref fields },
            ..
        }) => fields,
        _ => unimplemented!(),
    };

    let ty = builder_type(fields, &builder_ident);
    let constructor = builder_constructor(fields, ident, builder_ident);
    let setters = builder_setters(fields, builder_ident);
    let builder = builder_method(fields, ident, builder_ident);

    quote! {
        #ty
        #constructor
        #setters
        #builder
    }
}

fn builder_type(fields: &syn::FieldsNamed, builder_ident: &Ident) -> TokenStream {
    let optional_fields = fields.named.iter().map(|Field { ident, ty, .. }| {
        if let Some(_ty) = type_behind_option(ty) {
            quote! { #ident: #ty}
        } else {
            quote! { #ident: Option<#ty>}
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
    fields: &syn::FieldsNamed,
    ident: &Ident,
    builder_ident: &Ident,
) -> TokenStream {
    let field_initializers = fields
        .named
        .iter()
        .map(|Field { ident, .. }| quote! { #ident: None});

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

fn builder_setters(fields: &syn::FieldsNamed, builder_ident: &Ident) -> TokenStream {
    let field_setters = fields.named.iter().map(|Field { ident, ty, .. }| {
        let ty = if let Some(true_type) = type_behind_option(ty) {
            true_type
        } else {
            ty
        };

        quote! {
            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });

    quote! {
        impl #builder_ident {
            #(#field_setters)*
        }
    }
    .into()
}

fn builder_method(fields: &syn::FieldsNamed, ident: &Ident, builder_ident: &Ident) -> TokenStream {
    let build_fields = fields.named.iter().map(|Field { ident, ty, .. }| {
        if let Some(_ty) = type_behind_option(ty) {
            quote! {
                #ident: self.#ident.take()
            }
        } else {
            quote! {
                #ident: self.#ident.take().ok_or("#ident not set")?
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

fn type_behind_option(ty: &Type) -> Option<&Type> {
    let Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { ref segments, .. },
            ..
        }) = ty else {return None};

    let last_segment = segments.last()?;
    if last_segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(ref args) = last_segment.arguments else {
        return None;
    };

    if args.args.len() != 1 {
        return None;
    }

    let GenericArgument::Type(ty) = args.args.first()? else {
        return None;
    };

    Some(ty)
}

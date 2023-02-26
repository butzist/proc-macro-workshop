use crate::{field::BuilderField, input::Input};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub(crate) fn builder_ident(input: &Input) -> Ident {
    format_ident!("{}Builder", input.ident)
}

pub(crate) fn output_builder_type(input: &Input) -> TokenStream {
    let optional_fields = input.fields.iter().map(|field| match *field {
        BuilderField::Mandatory { ident, ty } | BuilderField::Optional { ident, ty } => {
            quote! { #ident: ::std::option::Option<#ty> }
        }
        BuilderField::Multi { ident, ty, .. } => {
            quote! { #ident: #ty }
        }
    });

    let builder_ident = builder_ident(input);

    quote! {
        pub struct #builder_ident {
            #(#optional_fields),*
        }
    }
    .into()
}

pub(crate) fn output_builder_constructor(input: &Input) -> TokenStream {
    let field_initializers = input.fields.iter().map(|field| match *field {
        BuilderField::Mandatory { ident, .. } | BuilderField::Optional { ident, .. } => {
            quote! { #ident: ::std::option::Option::None }
        }
        BuilderField::Multi { ident, .. } => {
            quote! { #ident: ::core::default::Default::default() }
        }
    });

    let ident = input.ident;
    let builder_ident = builder_ident(input);

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

pub(crate) fn output_setters(input: &Input) -> TokenStream {
    let builder_ident = builder_ident(input);

    let field_setters = input.fields.iter().flat_map(|field| match *field {
        BuilderField::Mandatory { ident, ty } | BuilderField::Optional { ident, ty } => {
            Some(quote! {
                fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = ::std::option::Option::Some(#ident);
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

    let multi_setters = input.fields.iter().flat_map(|field| match *field {
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

pub(crate) fn output_build_method(input: &Input) -> TokenStream {
    let build_fields = input.fields.iter().map(|field| match *field {
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

    let ident = input.ident;
    let builder_ident = builder_ident(input);

    quote! {
        impl #builder_ident {
            pub fn build(&mut self) -> ::std::result::Result<#ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                ::std::result::Result::Ok(#ident {
                    #(#build_fields),*
                })
            }
        }

    }
    .into()
}

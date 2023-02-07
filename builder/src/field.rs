use proc_macro2::Ident;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::Parse;
use syn::{
    Attribute, Error, Field, GenericArgument, LitStr, Path, PathArguments, Result, Token, Type,
};

pub(crate) enum BuilderField<'a> {
    Mandatory {
        ident: &'a Ident,
        ty: &'a Type,
    },
    Optional {
        ident: &'a Ident,
        ty: &'a Type,
    },
    Multi {
        ident: &'a Ident,
        attrs: Vec<Ident>,
        with_set_all: bool,
        ty: &'a Type,
        elem_ty: &'a Type,
    },
}

impl<'a> TryFrom<&'a Field> for BuilderField<'a> {
    type Error = Error;

    fn try_from(field: &'a Field) -> Result<Self> {
        let ident = field.ident.as_ref().unwrap();
        let each_attrs = get_each_attrs(field)?;

        if each_attrs.len() > 0 {
            let elem_ty = type_behind_vec(&field.ty).unwrap();
            let with_set_all = each_attrs.iter().all(|attr| attr != ident);

            Ok(BuilderField::Multi {
                ident,
                with_set_all,
                attrs: each_attrs,
                ty: &field.ty,
                elem_ty,
            })
        } else if let Some(ty) = type_behind_option(&field.ty) {
            Ok(BuilderField::Optional { ident, ty })
        } else {
            Ok(BuilderField::Mandatory {
                ident,
                ty: &field.ty,
            })
        }
    }
}

#[derive(Debug)]
struct EachArg {
    each: Ident,
    _equals_token: Token![=],
    alias: LitStr,
}

impl Parse for EachArg {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg = EachArg {
            each: input.parse()?,
            _equals_token: input.parse()?,
            alias: input.parse()?,
        };

        if arg.each.to_string() != "each" {
            return Err(Error::new_spanned(arg.each, "expected \"each\""));
        }

        Ok(arg)
    }
}

fn get_each_attrs(field: &Field) -> Result<Vec<Ident>> {
    field
        .attrs
        .iter()
        .filter_map(|attr| {
            if !path_is_ident(&attr.path, "builder") {
                return None;
            }

            Some(parse_each_attr(attr))
        })
        .collect()
}

fn parse_each_attr(attr: &Attribute) -> Result<Ident> {
    let arg: EachArg = attr.parse_args().map_err(|_| {
        let mut error_tokens = attr.tokens.clone();
        error_tokens.append_all(attr.path.segments.to_token_stream());
        Error::new_spanned(error_tokens, "expected `builder(each = \"...\")`")
    })?;
    Ok(Ident::new(&arg.alias.value(), arg.alias.span()))
}

fn path_is_ident(path: &Path, ident: &str) -> bool {
    let Some(path_ident) = path
            .get_ident() else {
                return false;
            };

    path_ident == ident
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

fn type_behind_vec(ty: &Type) -> Option<&Type> {
    let Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { ref segments, .. },
            ..
        }) = ty else {return None};

    let last_segment = segments.last()?;
    if last_segment.ident != "Vec" {
        return None;
    }

    let PathArguments::AngleBracketed(ref args) = last_segment.arguments else {
        return None;
    };

    if args.args.len() < 1 {
        return None;
    }

    let GenericArgument::Type(ty) = args.args.first()? else {
        return None;
    };

    Some(ty)
}

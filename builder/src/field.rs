use proc_macro2::Ident;
use syn::{Field, GenericArgument, PathArguments, Type};

pub(crate) enum BuilderField<'a> {
    Mandatory { ident: &'a Ident, ty: &'a Type },
    Optional { ident: &'a Ident, ty: &'a Type },
}

impl<'a> From<&'a Field> for BuilderField<'a> {
    fn from(field: &'a Field) -> Self {
        let ident = field.ident.as_ref().unwrap();
        if let Some(ty) = type_behind_option(&field.ty) {
            BuilderField::Optional { ident, ty }
        } else {
            BuilderField::Mandatory {
                ident,
                ty: &field.ty,
            }
        }
    }
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

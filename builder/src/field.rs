use proc_macro2::Ident;
use syn::{Field, GenericArgument, Lit, Meta, MetaList, MetaNameValue, Path, PathArguments, Type};

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

impl<'a> From<&'a Field> for BuilderField<'a> {
    fn from(field: &'a Field) -> Self {
        let ident = field.ident.as_ref().unwrap();
        let each_attrs = get_each_attrs(field);

        if each_attrs.len() > 0 {
            let elem_ty = type_behind_vec(&field.ty).unwrap();
            let with_set_all = each_attrs.iter().all(|attr| attr != ident);
            BuilderField::Multi {
                ident,
                with_set_all,
                attrs: each_attrs,
                ty: &field.ty,
                elem_ty,
            }
        } else if let Some(ty) = type_behind_option(&field.ty) {
            BuilderField::Optional { ident, ty }
        } else {
            BuilderField::Mandatory {
                ident,
                ty: &field.ty,
            }
        }
    }
}

fn get_each_attrs(field: &Field) -> Vec<Ident> {
    field
        .attrs
        .iter()
        .flat_map(|attr| {
            if !path_is_ident(&attr.path, "builder") {
                return vec![];
            }

            let meta = attr.parse_meta().unwrap();
            match meta {
                Meta::List(MetaList { nested, .. }, ..) => nested.into_iter(),
                _ => unimplemented!(),
            }
            .map(|meta| match meta {
                syn::NestedMeta::Meta(meta) => meta,
                _ => unimplemented!(),
            })
            .map(|meta| match meta {
                Meta::NameValue(MetaNameValue { path, lit, .. })
                    if path_is_ident(&path, "each") =>
                {
                    lit
                }
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>()
        })
        .map(|lit| match lit {
            Lit::Str(ref s) => Ident::new(&s.value(), lit.span()),
            _ => unimplemented!(),
        })
        .collect()
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

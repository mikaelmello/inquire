use quote::spanned::Spanned;
use syn::{Error, GenericArgument, PathArguments, Type, TypePath};

fn path_is_vec(path: &TypePath) -> bool {
    path.path.segments.len() == 1 && path.path.segments.iter().next().unwrap().ident == "Vec"
}

pub fn extract_type_from_vec(ty: &Type) -> Result<Type, Error> {
    match ty {
        Type::Path(typepath) if typepath.qself.is_none() && path_is_vec(typepath) => {
            // Get the first segment of the path (there is only one, in fact: "Option"):
            let type_params = &typepath.path.segments.first().unwrap().arguments;
            // It should have only on angle-bracketed param ("<String>"):
            let generic_arg = match type_params {
                PathArguments::AngleBracketed(params) => Ok(params.args.first().unwrap()),
                _ => Err(Error::new(
                    typepath.path.get_ident().unwrap().span(),
                    "expected `Vec<_>`",
                )),
            }?;
            // This argument must be a type:
            match generic_arg {
                GenericArgument::Type(ty) => Ok(ty.clone()),
                _ => Err(Error::new(
                    typepath.path.get_ident().unwrap().span(),
                    "expected `Vec<T>`",
                )),
            }
        }
        e => Err(Error::new(e.__span(), "expected `Vec<T>`")),
    }
}

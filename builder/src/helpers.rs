use proc_macro2::TokenStream;
use quote::TokenStreamExt;

pub(crate) trait CollectErrorsExt<T, E>: Iterator {
    fn collect_errors(self) -> Result<Vec<T>, Vec<E>>;
}

pub(crate) trait CollectErrorTokensExt<T>: Iterator {
    fn collect_errors_to_stream(self) -> Result<Vec<T>, TokenStream>;
}

impl<I, T, E> CollectErrorsExt<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn collect_errors(self) -> Result<Vec<T>, Vec<E>> {
        let (ok, err): (Vec<T>, Vec<E>) =
            self.fold(Default::default(), |(mut ok, mut err), res| {
                match res {
                    Ok(e) => ok.push(e),
                    Err(e) => err.push(e),
                };
                (ok, err)
            });

        if err.len() > 0 {
            Err(err)
        } else {
            Ok(ok)
        }
    }
}

impl<I, T> CollectErrorTokensExt<T> for I
where
    I: Iterator<Item = syn::Result<T>>,
{
    fn collect_errors_to_stream(self) -> Result<Vec<T>, TokenStream> {
        let result = self.collect_errors();

        result.map_err(|errs| {
            let mut tokens = TokenStream::new();
            for err in errs {
                tokens.append_all(err.to_compile_error())
            }

            tokens
        })
    }
}

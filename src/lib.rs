use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
};

use proc_macro::{Ident, TokenStream, TokenTree};
use quote::quote;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Mode {
    Read,
    ReadClone,
    Write,
    Lock,
}

impl TryFrom<Ident> for Mode {
    type Error = &'static str;

    fn try_from(value: Ident) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "read" => Ok(Self::Read),
            "readclone" => Ok(Self::ReadClone),
            "write" => Ok(Self::Write),
            "lock" => Ok(Self::Lock),
            _ => Err("mode must be read|readclone|write|lock"),
        }
    }
}

#[proc_macro]
pub fn sorted_locks(tokens: TokenStream) -> TokenStream {
    let mut tokens = tokens.into_iter();

    let mut v = vec![];

    loop {
        match (tokens.next(), tokens.next(), tokens.next()) {
            (Some(TokenTree::Ident(target)), Some(TokenTree::Ident(mode)), delim) => {
                if let Some(TokenTree::Punct(delim)) = delim {
                    if delim.as_char() != ',' {
                        panic!("Must be \"<target> <mode> [,]\"");
                    }
                }

                let mode: Mode = match mode.try_into() {
                    Ok(mode) => mode,
                    Err(err) => panic!("{}", err),
                };

                v.push((target, mode));
            }
            (None, None, None) => break,
            _ => panic!("Must be \"<target> <mode> [,]\""),
        }
    }

    v.sort_by(|lhs, rhs| {
        let lhs_ident = lhs.0.to_string();
        let rhs_ident = rhs.0.to_string();

        if lhs_ident < rhs_ident {
            Ordering::Less
        } else if lhs_ident > rhs_ident {
            Ordering::Greater
        } else {
            lhs.1.cmp(&rhs.1)
        }
    });

    let mut ts = proc_macro2::TokenStream::new();

    for (target, mode) in v {
        let target: TokenStream = TokenTree::Ident(target).into();
        let target: proc_macro2::TokenStream = target.into();

        ts.extend(match mode {
            Mode::Read => quote! { let #target = self.#target.read().expect("must read"); },
            Mode::ReadClone => {
                quote! { let #target = self.#target.read().expect("must read").clone(); }
            }
            Mode::Write => quote! { let mut #target = self.#target.write().expect("must write"); },
            Mode::Lock => quote! { let mut #target = self.#target.lock().expect("must get lock"); },
        });
    }

    ts.into()
}

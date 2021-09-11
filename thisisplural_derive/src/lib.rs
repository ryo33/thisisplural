use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, Ident, ItemStruct, Path, PathArguments, PathSegment, Type, TypePath};

#[proc_macro_derive(Plural)]
pub fn derive_plural(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = parse(input).unwrap();
    let plural_ident = item_struct.ident;
    let field = item_struct.fields.iter().next().expect("no field found");
    let field_ident = field
        .ident
        .as_ref()
        .map_or(quote![0], ToTokens::into_token_stream);
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = &field.ty
    {
        // last() for ignore paths such as "std::collections::"
        let segment = segments.iter().cloned().last().unwrap();
        if let PathSegment {
            ident,
            arguments: PathArguments::AngleBracketed(arguments),
        } = segment
        {
            let arguments = arguments
                .args
                .iter()
                .map(ToTokens::to_token_stream)
                .collect();
            let field = quote![#field];
            match ident.to_string().as_str() {
                "Vec" => return vec(plural_ident, field, arguments, field_ident).into(),
                "HashMap" => return hash_map(plural_ident, field, arguments, field_ident).into(),
                collection => panic!("{} is not supported yet", collection),
            }
        } else {
            panic!("not collection type is found");
        }
    } else {
        panic!("the first field should be a collection");
    }
}

fn vec(
    plural_ident: Ident,
    field: TokenStream,
    arguments: Vec<TokenStream>,
    accessor: TokenStream,
) -> TokenStream {
    assert!(arguments.len() == 1);
    let item = &arguments[0];

    let into = into(&plural_ident, &field, &accessor);
    let from = from(&plural_ident, &field);
    let deref = deref(&plural_ident, &field, &accessor);
    let deref_mut = deref_mut(&plural_ident, &accessor);
    let into_iter = into_iter(&plural_ident, &field, &item, &accessor);
    let from_iter = from_iter(&plural_ident, &item);
    let extend = extend(&plural_ident, &item, &accessor);
    let iter_and_iter_mut =
        iter_and_iter_mut(&plural_ident, &accessor, quote![&#item], quote![&mut #item]);
    quote! {
        #into
        #from
        #deref
        #deref_mut
        #into_iter
        #from_iter
        #iter_and_iter_mut
    }
}

fn hash_map(
    plural_ident: Ident,
    field: TokenStream,
    arguments: Vec<TokenStream>,
    accessor: TokenStream,
) -> TokenStream {
    assert!(arguments.len() == 2);
    let key = &arguments[0];
    let value = &arguments[1];
    let item = quote![(#key, #value)];

    let into = into(&plural_ident, &field, &accessor);
    let from = from(&plural_ident, &field);
    let deref = deref(&plural_ident, &field, &accessor);
    let deref_mut = deref_mut(&plural_ident, &accessor);
    let into_iter = into_iter(&plural_ident, &field, &item, &accessor);
    let from_iter = from_iter(&plural_ident, &item);
    let extend = extend(&plural_ident, &item, &accessor);
    let iter_and_iter_mut = iter_and_iter_mut(
        &plural_ident,
        &accessor,
        quote![(&#key, &#value)],
        quote![(&#key, &mut #value)],
    );
    quote! {
        #into
        #from
        #deref
        #deref_mut
        #into_iter
        #from_iter
        #extend
        #iter_and_iter_mut
    }
}

fn into_iter(
    plural_ident: &Ident,
    field: &TokenStream,
    item: &TokenStream,
    accessor: &TokenStream,
) -> TokenStream {
    quote! {
        impl IntoIterator for #plural_ident {
            type Item = #item;
            type IntoIter = <#field as IntoIterator>::IntoIter;
            fn into_iter(self) -> Self::IntoIter {
                self.#accessor.into_iter()
            }
        }
    }
}

fn from_iter(plural_ident: &Ident, item: &TokenStream) -> TokenStream {
    quote! {
        impl std::iter::FromIterator<#item> for #plural_ident {
            fn from_iter<T: IntoIterator<Item = #item>>(iter: T) -> Self {
                Self(iter.into_iter().collect())
            }
        }
    }
}

fn extend(plural_ident: &Ident, item: &TokenStream, accessor: &TokenStream) -> TokenStream {
    quote! {
        impl std::iter::Extend<#item> for #plural_ident {
            fn extend<T: IntoIterator<Item = #item>>(&mut self, iter: T) {
                self.#accessor.extend(iter);
            }
        }
    }
}

fn iter_and_iter_mut(
    plural_ident: &Ident,
    accessor: &TokenStream,
    item_ref: TokenStream,
    item_mut: TokenStream,
) -> TokenStream {
    quote! {
        impl #plural_ident {
            fn iter(&self) -> impl Iterator<Item = #item_ref> {
                self.#accessor.iter()
            }

            fn iter_mut(&mut self) -> impl Iterator<Item = #item_mut> {
                self.#accessor.iter_mut()
            }
        }
    }
}

fn into(plural_ident: &Ident, field: &TokenStream, accessor: &TokenStream) -> TokenStream {
    quote! {
        impl Into<#field> for #plural_ident {
            fn into(self) -> #field {
                self.#accessor
            }
        }
    }
}

fn from(plural_ident: &Ident, field: &TokenStream) -> TokenStream {
    quote! {
        impl From<#field> for #plural_ident {
            fn from(field: #field) -> Self {
                Self(field)
            }
        }
    }
}

fn deref(plural_ident: &Ident, field: &TokenStream, accessor: &TokenStream) -> TokenStream {
    quote! {
        impl std::ops::Deref for #plural_ident {
            type Target = #field;

            fn deref(&self) -> &#field {
                &self.#accessor
            }
        }
    }
}

fn deref_mut(plural_ident: &Ident, accessor: &TokenStream) -> TokenStream {
    quote! {
        impl std::ops::DerefMut for #plural_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#accessor
            }
        }
    }
}

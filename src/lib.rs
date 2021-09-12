use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse, ConstParam, Generics, Ident, ItemStruct, LifetimeDef, Path, PathArguments, PathSegment,
    Type, TypeParam, TypePath,
};

#[proc_macro_derive(Plural)]
pub fn derive_plural(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = parse(input).unwrap();
    let generics = item_struct.generics;
    let plural_ident = item_struct.ident;
    let field = item_struct.fields.iter().next().expect("no field found");
    let field_type = &field.ty;
    let field_ident = field
        .ident
        .as_ref()
        .map_or(quote![0], ToTokens::into_token_stream);
    let item = if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = &field_type
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
            match ident.to_string().as_str() {
                "Vec" => vec_item(arguments),
                "HashMap" => hash_map_item(arguments),
                collection => panic!("{} is not supported yet", collection),
            }
        } else {
            panic!("not collection type is found");
        }
    } else {
        panic!("the first field should be a collection");
    };

    let field_type = quote![#field_type];
    let into = impl_trait(&plural_ident, &generics, into(&field_type, &field_ident));
    let from = impl_trait(&plural_ident, &generics, from(&field_type));
    let deref = impl_trait(&plural_ident, &generics, deref(&field_type, &field_ident));
    let deref_mut = impl_trait(&plural_ident, &generics, deref_mut(&field_ident));
    let into_iter = impl_trait(
        &plural_ident,
        &generics,
        into_iter(&field_type, &item, &field_ident),
    );
    let from_iter = impl_trait(&plural_ident, &generics, from_iter(&item));
    proc_macro::TokenStream::from(quote! {
        #into
        #from
        #deref
        #deref_mut
        #into_iter
        #from_iter
    })
}

fn vec_item(mut arguments: Vec<TokenStream>) -> TokenStream {
    assert!(arguments.len() == 1);
    arguments.pop().unwrap()
}

fn hash_map_item(arguments: Vec<TokenStream>) -> TokenStream {
    assert!(arguments.len() == 2);
    let key = &arguments[0];
    let value = &arguments[1];
    quote![(#key, #value)]
}

fn into_iter(
    field: &TokenStream,
    item: &TokenStream,
    accessor: &TokenStream,
) -> (TokenStream, TokenStream) {
    (
        quote![IntoIterator],
        quote! {
            type Item = #item;
            type IntoIter = <#field as IntoIterator>::IntoIter;
            fn into_iter(self) -> Self::IntoIter {
                self.#accessor.into_iter()
            }
        },
    )
}

fn from_iter(item: &TokenStream) -> (TokenStream, TokenStream) {
    (
        quote![
        std::iter::FromIterator<#item>],
        quote! {
            fn from_iter<I: IntoIterator<Item = #item>>(iter: I) -> Self {
                Self(iter.into_iter().collect())
            }
        },
    )
}

fn into(field: &TokenStream, accessor: &TokenStream) -> (TokenStream, TokenStream) {
    (
        quote![Into<#field>],
        quote! {
            fn into(self) -> #field {
                self.#accessor
            }
        },
    )
}

fn from(field: &TokenStream) -> (TokenStream, TokenStream) {
    (
        quote![From<#field>],
        quote! {
            fn from(field: #field) -> Self {
                Self(field)
            }
        },
    )
}

fn deref(field: &TokenStream, accessor: &TokenStream) -> (TokenStream, TokenStream) {
    (
        quote![std::ops::Deref],
        quote! {
            type Target = #field;

            fn deref(&self) -> &#field {
                &self.#accessor
            }
        },
    )
}

fn deref_mut(accessor: &TokenStream) -> (TokenStream, TokenStream) {
    (
        quote![std::ops::DerefMut],
        quote! {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#accessor
            }
        },
    )
}

fn impl_trait(
    plural_ident: &Ident,
    generics: &Generics,
    (trait_, content): (TokenStream, TokenStream),
) -> TokenStream {
    let generics_without_bounds = generics.params.iter().map(|param| match param {
        syn::GenericParam::Type(TypeParam { ident, .. }) => ident.to_token_stream(),
        syn::GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => lifetime.to_token_stream(),
        syn::GenericParam::Const(ConstParam { ident, .. }) => ident.to_token_stream(),
    });
    quote! {
        impl#generics #trait_ for #plural_ident<#(#generics_without_bounds,)*> {
            #content
        }
    }
}

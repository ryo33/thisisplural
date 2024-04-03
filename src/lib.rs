use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse, parse_quote, spanned::Spanned as _, ConstParam, GenericArgument, GenericParam, Generics,
    Ident, ItemStruct, LifetimeParam, Path, PathArguments, PathSegment, Type, TypeParam, TypePath,
};

#[proc_macro_derive(Plural)]
pub fn derive_plural(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = parse(input).unwrap();
    let generics = &item_struct.generics;
    let generics_without_bounds = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(TypeParam { ident, .. }) => ident.to_token_stream(),
            syn::GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => {
                lifetime.to_token_stream()
            }
            syn::GenericParam::Const(ConstParam { ident, .. }) => ident.to_token_stream(),
        })
        .collect::<Vec<_>>();
    let ident = &item_struct.ident;
    let Some((field, field_ident)) = item_struct.fields.iter().next().map(|field| {
        (
            field,
            field
                .ident
                .as_ref()
                .map_or(quote![0], ToTokens::into_token_stream),
        )
    }) else {
        return quote_spanned!(item_struct.span() => compile_error!("expected a field")).into();
    };
    let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = &field.ty
    else {
        return quote_spanned!(field.ty.span() => compile_error!("expected a collection")).into();
    };
    // last() for ignore paths such as "std::collections::"
    let segment = segments.iter().last().unwrap();
    let PathSegment {
        ident: _collection_name,
        arguments: PathArguments::AngleBracketed(arguments),
    } = segment
    else {
        return quote_spanned!(segment.span() => compile_error!("expected a collection")).into();
    };
    if arguments.args.is_empty() {
        return quote_spanned!(segment.span() => compile_error!("failed to get the item type for this collection")).into();
    }
    let item = if arguments.args.len() >= 2 {
        let key = &arguments.args[0];
        let value = &arguments.args[1];
        Item::KeyValue { key, value }
    } else {
        let item = &arguments.args[0];
        Item::Value(item)
    };

    let plural = Plural {
        ident,
        generics,
        generics_without_bounds,
        field_ident,
        collection: &field.ty,
        item,
    };

    let into = plural.impl_trait(plural.into_());
    let from = plural.impl_trait(plural.from());
    let into_iter = plural.impl_trait(plural.into_iter());
    let into_iter_ref = plural.into_iter_ref();
    let from_iter = plural.impl_trait(plural.from_iter());
    let extend = plural.impl_trait(plural.extend());
    let delegate = plural.delegate(plural.methods());

    proc_macro::TokenStream::from(quote! {
        #into
        #from
        #into_iter
        #into_iter_ref
        #from_iter
        #extend
        #delegate
    })
}

enum Item<'a> {
    KeyValue {
        key: &'a GenericArgument,
        value: &'a GenericArgument,
    },
    Value(&'a GenericArgument),
}

impl Item<'_> {
    pub fn reference(&self, lifetime: impl ToTokens) -> TokenStream {
        match self {
            Item::KeyValue { key, value } => quote![(& #lifetime #key, & #lifetime #value)],
            Item::Value(item) => quote![& #lifetime #item],
        }
    }
}

impl ToTokens for Item<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item = match self {
            Item::KeyValue { key, value } => quote![(#key, #value)],
            Item::Value(item) => quote![#item],
        };
        tokens.extend(item);
    }
}

struct Plural<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
    generics_without_bounds: Vec<TokenStream>,
    collection: &'a syn::Type,
    field_ident: TokenStream,
    item: Item<'a>,
}

impl Plural<'_> {
    fn into_iter(&self) -> (TokenStream, TokenStream) {
        let Plural {
            field_ident,
            collection,
            item: item_type,
            ..
        } = self;
        (
            quote![IntoIterator],
            quote! {
                type Item = #item_type;
                type IntoIter = <#collection as IntoIterator>::IntoIter;
                fn into_iter(self) -> Self::IntoIter {
                    self.#field_ident.into_iter()
                }
            },
        )
    }

    fn into_iter_ref(&self) -> TokenStream {
        let Plural {
            ident,
            field_ident,
            collection,
            item,
            generics,
            generics_without_bounds,
            ..
        } = self;
        let lifetime: GenericParam = parse_quote!('plural);
        let mut generics = (*generics).to_owned();
        generics.params.insert(0, lifetime.clone());
        let item_type = item.reference(&lifetime);
        quote! {
            impl #generics IntoIterator for & #lifetime #ident<#(#generics_without_bounds,)*> {
                type Item = #item_type;
                type IntoIter = <& #lifetime #collection as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    self.#field_ident.iter()
                }
            }
        }
    }

    fn from_iter(&self) -> (TokenStream, TokenStream) {
        let Plural {
            item: item_type, ..
        } = self;
        (
            quote![
        std::iter::FromIterator<#item_type>],
            quote! {
                fn from_iter<I: IntoIterator<Item = #item_type>>(iter: I) -> Self {
                    Self(iter.into_iter().collect())
                }
            },
        )
    }

    fn into_(&self) -> (TokenStream, TokenStream) {
        let Plural {
            field_ident,
            collection,
            ..
        } = self;
        (
            quote![Into<#collection>],
            quote! {
                fn into(self) -> #collection {
                    self.#field_ident
                }
            },
        )
    }

    fn from(&self) -> (TokenStream, TokenStream) {
        let Plural { collection, .. } = self;
        (
            quote![From<#collection>],
            quote! {
                fn from(field: #collection) -> Self {
                    Self(field)
                }
            },
        )
    }

    fn extend(&self) -> (TokenStream, TokenStream) {
        let Plural {
            field_ident,
            item: item_type,
            ..
        } = self;
        (
            quote![std::iter::Extend<#item_type>],
            quote! {
                fn extend<I: IntoIterator<Item = #item_type>>(&mut self, iter: I) {
                    self.#field_ident.extend(iter)
                }
            },
        )
    }

    fn impl_trait(&self, (trait_, content): (TokenStream, TokenStream)) -> TokenStream {
        let Plural {
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote! {
            impl #generics #trait_ for #ident<#(#generics_without_bounds,)*> {
                #content
            }
        }
    }

    fn methods(&self) -> TokenStream {
        let Plural {
            field_ident,
            item,
            collection,
            ..
        } = self;
        let reference = item.reference(quote! {});
        quote! {
            /// Returns the number of elements in the collection.
            pub fn len(&self) -> usize {
                self.#field_ident.len()
            }

            /// Returns `true` if the collection contains no elements.
            pub fn is_empty(&self) -> bool {
                self.#field_ident.is_empty()
            }

            /// Iterates over the collection.
            pub fn iter(&self) -> impl Iterator<Item = #reference> {
                self.#field_ident.iter()
            }

            /// Returns the capacity of the collection.
            pub fn capacity(&self) -> usize {
                self.#field_ident.capacity()
            }

            /// Reserves capacity for at least `additional` more elements to be inserted in the collection.
            pub fn reserve(&mut self, additional: usize) {
                self.#field_ident.reserve(additional)
            }

            /// Construct a new empty collection with the specified capacity.
            pub fn with_capacity(capacity: usize) -> Self {
                Self { #field_ident : <#collection>::with_capacity(capacity) }
            }
        }
    }

    fn delegate(&self, content: TokenStream) -> TokenStream {
        let Plural {
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote! {
            impl #generics #ident<#(#generics_without_bounds,)*> {
                #content
            }
        }
    }
}

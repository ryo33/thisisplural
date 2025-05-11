use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{self, Parse},
    parse_quote,
    spanned::Spanned as _,
    ConstParam, GenericArgument, GenericParam, Generics, Ident, ItemStruct, LifetimeParam, Path,
    PathArguments, PathSegment, Type, TypeParam, TypePath,
};

struct Methods {
    methods: Vec<(syn::Ident, Method)>,
}

enum Method {
    Len,
    IsEmpty,
    Iter,
    Capacity,
    Reserve,
    WithCapacity,
    Extend,
    New,
    Clear,
    FromPlural,
    FromInner,
    IntoIter,
    FromIter,
    IntoIterRef,
}

impl Method {
    fn from_ident(ident: &syn::Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "len" => Ok(Method::Len),
            "is_empty" => Ok(Method::IsEmpty),
            "iter" => Ok(Method::Iter),
            "capacity" => Ok(Method::Capacity),
            "reserve" => Ok(Method::Reserve),
            "with_capacity" => Ok(Method::WithCapacity),
            "extend" => Ok(Method::Extend),
            "new" => Ok(Method::New),
            "clear" => Ok(Method::Clear),
            "from_plural" => Ok(Method::FromPlural),
            "from_inner" => Ok(Method::FromInner),
            "into_iter" => Ok(Method::IntoIter),
            "from_iter" => Ok(Method::FromIter),
            "into_iter_ref" => Ok(Method::IntoIterRef),
            _ => Err(syn::Error::new(ident.span(), "invalid method")),
        }
    }
}

impl Parse for Methods {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let parsed_methods = input.parse_terminated(syn::Ident::parse, syn::Token![,])?;
        let mut methods = Vec::new();
        for ident in parsed_methods {
            methods.push((ident.clone(), Method::from_ident(&ident)?));
        }
        Ok(Methods { methods })
    }
}

#[proc_macro_derive(Plural, attributes(plural))]
/// If `#[plural(len, is_empty, iter)]` is specified, only the specified methods will be implemented.
/// Available methods:
/// - `len`
/// - `is_empty`
/// - `iter`
/// - `capacity`
/// - `reserve`
/// - `with_capacity`
/// - `new`
/// - `clear`
/// - `extend` (provides `impl Extend<ItemType>`)
/// - `from_inner` (provides `impl From<InnerCollectionType> for NewType`)
/// - `from_plural` (provides `impl From<NewType> for InnerCollectionType`)
/// - `into_iter` (provides `impl IntoIterator` for `Self`)
/// - `from_iter` (provides `impl FromIterator<ItemType>`)
/// - `into_iter_ref` (provides `impl IntoIterator for &Self`)
pub fn derive_plural(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_struct: ItemStruct = syn::parse_macro_input!(input as ItemStruct);
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
    let segment = segments.iter().next_back().unwrap();
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

    let mut methods = vec![];
    for attr in item_struct
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("plural"))
    {
        let meta = match attr.meta.require_list() {
            Ok(meta) => meta,
            Err(e) => {
                return e.into_compile_error().into();
            }
        };
        let parsed = match meta.parse_args::<Methods>() {
            Ok(methods) => methods,
            Err(e) => {
                return e.into_compile_error().into();
            }
        };
        methods.extend(parsed.methods);
    }

    let plural = Plural {
        ident,
        generics,
        generics_without_bounds,
        field_ident,
        collection: &field.ty,
        item,
    };

    if methods.is_empty() {
        let span = plural.ident.span();

        let from_plural_impl = plural.from(span);
        let from_inner_impl = plural.from_inner_def(span);
        let into_iter_impl = plural.into_iter(span);
        let into_iter_ref_impl = plural.into_iter_ref(span);
        let from_iter_impl = plural.from_iter(span);
        let extend_impl = plural.extend(span);

        let mut all_method_definitions = TokenStream::new();
        all_method_definitions.extend(plural.len_def(span));
        all_method_definitions.extend(plural.is_empty_def(span));
        all_method_definitions.extend(plural.iter_def(span));
        all_method_definitions.extend(plural.capacity_def(span));
        all_method_definitions.extend(plural.reserve_def(span));
        all_method_definitions.extend(plural.with_capacity_def(span));
        all_method_definitions.extend(plural.new_def(span));
        all_method_definitions.extend(plural.clear_def(span));

        let delegate_impl = plural.delegate(all_method_definitions);

        quote! {
            #from_plural_impl
            #from_inner_impl
            #into_iter_impl
            #into_iter_ref_impl
            #from_iter_impl
            #extend_impl
            #delegate_impl
        }
        .into()
    } else {
        let mut individual_method_definitions = TokenStream::new();
        let mut trait_implementations = TokenStream::new();

        for (method_ident, method_enum_variant) in methods {
            let span = method_ident.span();
            match method_enum_variant {
                Method::Len => {
                    individual_method_definitions.extend(plural.len_def(span));
                }
                Method::IsEmpty => {
                    individual_method_definitions.extend(plural.is_empty_def(span));
                }
                Method::Iter => {
                    individual_method_definitions.extend(plural.iter_def(span));
                }
                Method::Capacity => {
                    individual_method_definitions.extend(plural.capacity_def(span));
                }
                Method::Reserve => {
                    individual_method_definitions.extend(plural.reserve_def(span));
                }
                Method::WithCapacity => {
                    individual_method_definitions.extend(plural.with_capacity_def(span));
                }
                Method::New => {
                    individual_method_definitions.extend(plural.new_def(span));
                }
                Method::Clear => {
                    individual_method_definitions.extend(plural.clear_def(span));
                }
                Method::Extend => {
                    trait_implementations.extend(plural.extend(span));
                }
                Method::FromPlural => {
                    trait_implementations.extend(plural.from(span));
                }
                Method::FromInner => {
                    trait_implementations.extend(plural.from_inner_def(span));
                }
                Method::IntoIter => {
                    trait_implementations.extend(plural.into_iter(span));
                }
                Method::FromIter => {
                    trait_implementations.extend(plural.from_iter(span));
                }
                Method::IntoIterRef => {
                    trait_implementations.extend(plural.into_iter_ref(span));
                }
            }
        }

        let mut final_code = TokenStream::new();
        if !individual_method_definitions.is_empty() {
            final_code.extend(plural.delegate(individual_method_definitions));
        }
        final_code.extend(trait_implementations);

        final_code.into()
    }
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
    fn into_iter(&self, span: proc_macro2::Span) -> TokenStream {
        let Plural {
            field_ident,
            collection,
            item: item_type,
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote_spanned! { span =>
            impl #generics IntoIterator for #ident<#(#generics_without_bounds,)*> {
                type Item = #item_type;
                type IntoIter = <#collection as IntoIterator>::IntoIter;
                fn into_iter(self) -> Self::IntoIter {
                    self.#field_ident.into_iter()
                }
            }
        }
    }

    fn into_iter_ref(&self, span: proc_macro2::Span) -> TokenStream {
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
        quote_spanned! { span =>
            impl #generics IntoIterator for & #lifetime #ident<#(#generics_without_bounds,)*> {
                type Item = #item_type;
                type IntoIter = <& #lifetime #collection as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    self.#field_ident.iter()
                }
            }
        }
    }

    fn from_iter(&self, span: proc_macro2::Span) -> TokenStream {
        let Plural {
            item: item_type,
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote_spanned! { span =>
            impl #generics core::iter::FromIterator<#item_type> for #ident<#(#generics_without_bounds,)*> {
                fn from_iter<I: IntoIterator<Item = #item_type>>(iter: I) -> Self {
                    Self(iter.into_iter().collect())
                }
            }
        }
    }

    fn from(&self, span: proc_macro2::Span) -> TokenStream {
        let Plural {
            ident,
            generics,
            generics_without_bounds,
            collection,
            field_ident,
            ..
        } = self;
        let new_type_full = quote! { #ident<#(#generics_without_bounds,)*> };
        quote_spanned! { span =>
            impl #generics From<#new_type_full> for #collection {
                fn from(new_type_instance: #new_type_full) -> #collection {
                    new_type_instance.#field_ident
                }
            }
        }
    }

    fn from_inner_def(&self, span: proc_macro2::Span) -> TokenStream {
        let Plural {
            collection,
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote_spanned! { span =>
            impl #generics From<#collection> for #ident<#(#generics_without_bounds,)*> {
                fn from(field: #collection) -> Self {
                    Self(field)
                }
            }
        }
    }

    fn extend(&self, span: proc_macro2::Span) -> TokenStream {
        let Plural {
            field_ident,
            item: item_type,
            ident,
            generics,
            generics_without_bounds,
            ..
        } = self;
        quote_spanned! { span =>
            impl #generics core::iter::Extend<#item_type> for #ident<#(#generics_without_bounds,)*> {
                fn extend<I: IntoIterator<Item = #item_type>>(&mut self, iter: I) {
                    self.#field_ident.extend(iter)
                }
            }
        }
    }

    fn len_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural { field_ident, .. } = self;
        let len = Ident::new("len", method_span);
        quote! {
            #[doc = "Returns the number of elements in the collection."]
            pub fn #len(&self) -> usize {
                self.#field_ident.len()
            }
        }
    }

    fn is_empty_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural { field_ident, .. } = self;
        let is_empty = Ident::new("is_empty", method_span);
        quote! {
            #[doc = "Returns `true` if the collection contains no elements."]
            pub fn #is_empty(&self) -> bool {
                self.#field_ident.is_empty()
            }
        }
    }

    fn iter_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural {
            field_ident, item, ..
        } = self;
        let iter = Ident::new("iter", method_span);
        let reference = item.reference(quote! {});
        quote! {
            /// Iterates over the collection.
            pub fn #iter(&self) -> impl Iterator<Item = #reference> {
                self.#field_ident.iter()
            }
        }
    }

    fn capacity_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural { field_ident, .. } = self;
        let capacity = Ident::new("capacity", method_span);
        quote! {
            #[doc = "Returns the capacity of the collection."]
            pub fn #capacity(&self) -> usize {
                self.#field_ident.capacity()
            }
        }
    }

    fn reserve_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural { field_ident, .. } = self;
        let reserve = Ident::new("reserve", method_span);
        quote! {
            #[doc = "Reserves capacity for at least `additional` more elements to be inserted in the collection."]
            pub fn #reserve(&mut self, additional: usize) {
                self.#field_ident.reserve(additional)
            }
        }
    }

    fn with_capacity_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural {
            field_ident,
            collection,
            ..
        } = self;
        let with_capacity = Ident::new("with_capacity", method_span);
        quote! {
            #[doc = "Construct a new empty collection with the specified capacity."]
            pub fn #with_capacity(capacity: usize) -> Self {
                #[allow(clippy::init_numbered_fields)]
                Self { #field_ident: <#collection>::with_capacity(capacity) }
            }
        }
    }

    fn new_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural {
            field_ident,
            collection,
            ..
        } = self;
        let new = Ident::new("new", method_span);
        quote! {
            #[doc = "Creates a new, empty collection."]
            pub fn #new() -> Self {
                #[allow(clippy::init_numbered_fields)]
                Self { #field_ident: <#collection>::new() }
            }
        }
    }

    fn clear_def(&self, method_span: proc_macro2::Span) -> TokenStream {
        let Plural { field_ident, .. } = self;
        let clear = Ident::new("clear", method_span);
        quote! {
            #[doc = "Clears the collection, removing all values."]
            pub fn #clear(&mut self) {
                self.#field_ident.clear()
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

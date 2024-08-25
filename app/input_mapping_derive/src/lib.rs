use proc_macro2::{Literal, TokenStream};
use quote::ToTokens;
use syn::{parse_macro_input, Fields, ItemEnum, Meta, MetaNameValue, Variant};

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;

#[proc_macro_derive(InputMapping, attributes(key, description))]
pub fn derive_mapping(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_enum = parse_macro_input!(input as ItemEnum);
    let trait_impl = generate_trait_impl(item_enum);

    proc_macro::TokenStream::from(trait_impl)
}

// TODO: Add check that mappings don't overlap.
fn generate_trait_impl(item_enum: ItemEnum) -> TokenStream {
    let mapping_entries = item_enum
        .variants
        .iter()
        .filter(|variant| matches!(variant.fields, Fields::Unit))
        .map(generate_mapping_entry);

    let mapping_constructors = mapping_entries.clone().map(|entry| {
        let key = entry.key;
        let description = entry.description;
        let event = entry.event;

        quote! {
            ::input_mapping_common::MappingEntry {
                key: ::ratatui::crossterm::event::KeyCode::Char(#key),
                description: (#description).to_string(),
                event: Self:: #event
            }
        }
    });

    let mapping_matchers = mapping_entries.map(|entry| {
        let key = entry.key;
        let event = entry.event;

        quote! {
            () if event.is_key_pressed(::ratatui::crossterm::event::KeyCode::Char(#key)) => ::std::option::Option::Some(Self:: #event)
        }
    });

    let ident = item_enum.ident;

    quote! {
        impl ::input_mapping_common::InputMappingT for #ident {
            fn get_mapping(&self) -> ::input_mapping_common::InputMapping<Self> {
                ::input_mapping_common::InputMapping {
                    // TODO: Concat flattened
                    mapping: vec![
                        #(#mapping_constructors,)*
                    ]
                }
            }

            fn map_event(&self, event: ::ratatui::crossterm::event::Event) -> ::std::option::Option<Self> {
                // TODO: Concat flattened
                match () {
                    #(#mapping_matchers,)*
                    _ => None,
                }
            }
        }
    }
}

struct MappingEntry {
    key: TokenStream,
    description: TokenStream,
    event: TokenStream,
}

fn generate_mapping_entry(variant: &Variant) -> MappingEntry {
    let mut key: Option<TokenStream> = None;
    let mut description: Option<TokenStream> = None;

    for attr in &variant.attrs {
        match &attr.meta {
            Meta::NameValue(MetaNameValue { path, value, .. }) => {
                if path.is_ident("key") {
                    if key.is_some() {
                        panic!("Duplicate definition for attribute: key");
                    }

                    key = Some(value.into_token_stream());
                } else if path.is_ident("description") {
                    if description.is_some() {
                        panic!("Duplicate definition for attribute: description");
                    }

                    description = Some(value.into_token_stream());
                }
            }
            _ => continue,
        }
    }

    let key = key.unwrap_or_else(|| {
        let key = variant
            .ident
            .to_string()
            .chars()
            .next()
            .expect("Non-empty identifier expected");

        Literal::character(key).into_token_stream()
    });

    let description = description.unwrap_or_else(|| Literal::string("").into_token_stream());

    MappingEntry {
        key,
        description,
        event: variant.ident.to_token_stream(),
    }
}

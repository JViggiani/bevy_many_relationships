use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for defining many-to-many relationship marker types.
///
/// This generates the marker trait implementation required by the
/// `bevy_many_relationships` runtime. The struct must be a unit struct.
///
/// # Example
///
/// ```ignore
/// #[derive(ManyRelationship)]
/// struct KnownContact;
/// ```
#[proc_macro_derive(ManyRelationship)]
pub fn derive_many_relationship(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::bevy_many_relationships::ManyRelationshipType for #name #ty_generics #where_clause {
            fn relationship_name() -> &'static str {
                stringify!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}

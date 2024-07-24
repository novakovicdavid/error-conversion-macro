use proc_macro::TokenStream;

use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Fields, FieldsUnnamed, Ident, parse_macro_input, Variant};
use syn::punctuated::Punctuated;
use syn::token::Comma;

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

/// Macro for deriving the `From` trait implementation for an enum with error variants.
/// The macro generates conversions from inner error types to the enum's variants.
///
/// # Attributes
/// - `without_anyhow`: Skips conversion for variants whose inner type do not have a variant containing an `anyhow::Error`.
///
/// # Example
/// ```rust
/// # mod anyhow {
/// #   pub struct Error;
/// # }
///
/// use anyhow::Error;
/// use error_conversion_macro::ErrorEnum;
///
/// #[derive(ErrorEnum)]
/// enum MyError {
///     OtherError(anyhow::Error),
///
///     #[without_anyhow]
///     CustomError(String),
/// }
/// ```
#[proc_macro_derive(ErrorEnum, attributes(without_anyhow))]
pub fn generate_from_impls(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    // get enum name
    let enum_name = &input.ident;
    let data = &input.data;

    // Validate that ErrorEnum is only implemented for enums
    let enum_data = match data {
        Data::Enum(data_enum) => data_enum,
        _ => return derive_error!("ErrorEnum is only implemented for enums"),
    };

    // A vector to store the generated impl From tokens
    let mut generated_tokens = Vec::new();

    // Find enum variant with anyhow::Error type
    let anyhow_variant = match get_variant_with_type(&enum_data.variants, "anyhow :: Error") {
        Some(variant) => variant,
        None => return derive_error!("Could not find a variant with anyhow::Error type in this enum")
    };

    // Generate impls
    for variant in &enum_data.variants {
        if &variant.ident == anyhow_variant {
            continue;
        }

        let token_stream = generate_impl(enum_name, variant, anyhow_variant);
        if let Some(stream) = token_stream {
            generated_tokens.push(stream);
        }
    }

    quote! {
        #(#generated_tokens)*
        impl From<anyhow::Error> for #enum_name {
            fn from(value: anyhow::Error) -> Self {
                #enum_name::#anyhow_variant(value.into())
            }
        }
    }.into()
}

fn get_unnamed_field(variant: &Variant) -> Option<&FieldsUnnamed> {
    match &variant.fields {
        Fields::Unnamed(field) => Some(field),
        _ => None
    }
}

fn get_variant_with_type<'a>(variants: &'a Punctuated<Variant, Comma>, with_type: &str) -> Option<&'a Ident> {
    variants.iter().find_map(|variant| {
        if let Some(field) = get_unnamed_field(variant) {
            let variant_name = &variant.ident;
            let variant_inner_type = &field.unnamed;
            let variant_inner_type_str = variant_inner_type.into_token_stream().to_string();

            if &*variant_inner_type_str == with_type {
                return Some(variant_name);
            }
        }

        None
    })
}

fn generate_impl(enum_name: &Ident, variant: &Variant, anyhow_variant: &Ident) -> Option<TokenStream2> {
    let field = match get_unnamed_field(variant) {
        Some(field) => field,
        None => return None,
    };

    let variant_name = &variant.ident;
    let variant_inner_type = &field.unnamed;

    // Check for the presence of `without_anyhow` attribute
    let without_anyhow_attribute = variant
        .attrs
        .iter()
        .find(|attr| attr.meta.clone().into_token_stream().to_string() == "without_anyhow");

    match without_anyhow_attribute {
        // Generate the full From implementation, extracting anyhow::Error from the variant type.
        None => quote! {
                    impl From<#variant_inner_type> for #enum_name {
                        fn from(value: #variant_inner_type) -> Self {
                            match value {
                                #variant_inner_type::#anyhow_variant(e) => #enum_name::#anyhow_variant(e),
                                _ => #enum_name::#variant_name(value),
                            }
                        }
                    }
                }.into(),

        // Don't extract anyhow::Error from the variant type, instead just wrap the type in our enum.
        Some(_) => quote! {
                    impl From<#variant_inner_type> for #enum_name {
                        fn from(value: #variant_inner_type) -> Self {
                            Self::#variant_name(value)
                        }
                    }
                }.into()
    }
}
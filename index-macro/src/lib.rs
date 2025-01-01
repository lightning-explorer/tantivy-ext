use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, LitStr, Token};

const TANTIVY_EXT_TYPES: [&str; 6] = ["Tokenized", "Str", "FastU64", "U64", "Date", "Score"];

#[proc_macro_derive(Index, attributes(tantivy_ext))]
pub fn derive_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let struct_fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named,
            _ => panic!("Index macro only supports named fields"),
        },
        _ => panic!("Index macro only supports structs"),
    };

    let ext_types = get_tantivy_ext_types(&struct_fields);
    let ext_types_tokens = ident_vec_comma_separated(ext_types);

    let mut schema_lines = Vec::new();
    let mut as_doc_lines = Vec::new();
    let mut from_doc_lines = Vec::new();

    let mut primary_key_line = None;

    for field in &struct_fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type: &syn::Type = &field.ty;

        if let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("tantivy_ext"))
        {
            let tokens = attr
                .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
                .unwrap();
            let token_first: String = tokens.first().unwrap().value();

            if token_first == "primary_key" {
                primary_key_line = Some(field_as_term(field_type, &field_name));
            }
        }

        if let Some(schema_field) = schema_field_from_type(field_type, &field_name_str){
            schema_lines.push(schema_field);
            as_doc_lines.push(quote! {
                schema.get_field(#field_name_str).unwrap() => self.#field_name.tantivy_val(),
            });
        }
        from_doc_lines.push(field_from_doc(field_type, &field_name));
    }

    let primary_key_impl = primary_key_line.unwrap_or_else(|| {
        panic!(
            "Primary key not specified. Consider annotating a field with `#[\"tantivy_ext(primary_key)\"]`"
        )
    });

    let expanded = quote! {
        impl Index for #struct_name {
            fn schema() -> tantivy::schema::Schema {
                let mut schema_builder = tantivy::schema::Schema::builder();
                #(#schema_lines)*
                schema_builder.build()
            }

            fn get_primary_key(&self) -> tantivy::Result<tantivy::Term> {
                #primary_key_impl
                Ok(term)
            }

            fn as_document(&self) -> tantivy::TantivyDocument {
                let schema = Self::schema();
                doc! {
                    #(#as_doc_lines)*
                }
            }

            fn from_document(doc:tantivy::TantivyDocument, score: f32)->Self{
                let schema = &#struct_name::schema();
                #(#from_doc_lines)*
                #struct_name {
                    #ext_types_tokens
                }
            }

            fn index_builder(path: PathBuf) -> SearchIndexBuilder<Self>
            where
                Self: std::marker::Sized,
            {
                SearchIndexBuilder::new(path)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Used for getting the primary key term
fn field_as_term(ty: &syn::Type, field_name: &syn::Ident) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_str = &segment.ident.to_string();
            match type_str.as_str() {
                "Tokenized" | "Str" => {
                    quote! {
                        let term = tantivy::Term::from_field_text(
                            Self::schema().get_field(stringify!(#field_name))?,
                            &self.#field_name.tantivy_val(),
                        );
                    }
                }
                "FastU64" | "U64" => {
                    quote! {
                        let term = tantivy::Term::from_field_u64(
                            Self::schema().get_field(stringify!(#field_name))?,
                            self.#field_name.tantivy_val(),
                        );
                    }
                }
                "Date" => {
                    quote! {
                        let term = tantivy::Term::from_field_date(
                            Self::schema().get_field(stringify!(#field_name))?,
                            self.#field_name.tantivy_val(),
                        );
                    }
                }
                _ => panic!("Unsupported primary key type: {}", type_str),
            }
        } else {
            panic!("Invalid type path");
        }
    } else {
        panic!(
            "Unsupported primary key type: {:?}",
            ty.to_token_stream().to_string()
        );
    }
}

fn field_from_doc(ty: &syn::Type, field_name: &syn::Ident) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_str = &segment.ident.to_string();
            match type_str.as_str() {
                "Tokenized" | "Str" => {
                    quote! {
                        let #field_name = util::field_extractor::field_as_string(schema, &doc,stringify!(#field_name))
                        .expect(&format!("Could not extract {} field from document",stringify!(#field_name)));
                    }
                }
                "FastU64" | "U64" => {
                    quote! {
                        let #field_name = util::field_extractor::field_as_u64(schema, &doc,stringify!(#field_name))
                        .expect(&format!("Could not extract {} field from document",stringify!(#field_name)));
                    }
                }
                "Date" => {
                    quote! {
                        let #field_name = util::field_extractor::field_as_date(schema, &doc,stringify!(#field_name))
                        .expect(&format!("Could not extract {} field from document",stringify!(#field_name)));
                    }
                }
                "Score" => {
                    quote!{
                        let #field_name = score;
                    }
                }
                _ => panic!("Unsupported4 field type: {}", type_str),
            }
        } else {
            panic!("Invalid type path");
        }
    } else {
        panic!(
            "Unsupported3 field type: {:?}",
            ty.to_token_stream().to_string()
        );
    }
}

/// Returns the necesssary tokens to register the provided field in the schema
fn schema_field_from_type(ty: &syn::Type, field_name: &str) -> Option<proc_macro2::TokenStream> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // Get the last path segment. Example: `fields::FastU64` -> `FastU64`
            let type_str = &segment.ident.to_string();
            match type_str.as_str() {
                "Tokenized" => {
                    Some(quote! { schema_builder.add_text_field(#field_name, tantivy::schema::TEXT | tantivy::schema::STORED); })
                }
                "Str" => {
                    Some(quote! { schema_builder.add_text_field(#field_name, tantivy::schema::STRING | tantivy::schema::STORED); })
                }
                "FastU64" => {
                    Some(quote! { schema_builder.add_u64_field(#field_name, tantivy::schema::FAST | tantivy::schema::STORED); })
                }
                "Date" => {
                    Some(quote! { schema_builder.add_date_field(#field_name, tantivy::schema::INDEXED | tantivy::schema::STORED); })
                }
                _ => None, // Unknown field. Don't include it in the schema
            }
        } else {
            panic!("Invalid type path");
        }
    } else {
        panic!(
            "Unsupported2 field type: {:?}",
            ty.to_token_stream().to_string()
        );
    }
}

fn get_tantivy_ext_types(
    struct_fields: &syn::FieldsNamed,
) -> Vec<(&proc_macro2::Ident, &syn::Type)> {
    let mut res = Vec::new();
    for field in &struct_fields.named {
        let field_name_ident = field.ident.as_ref().unwrap();
        let ty: &syn::Type = &field.ty;
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                let type_str = segment.ident.to_string();
                if TANTIVY_EXT_TYPES.contains(&type_str.as_str()) {
                    res.push((field_name_ident, ty));
                }
            } else {
                panic!("Invalid type path");
            }
        } else {
            panic!(
                "Unsupported1 field type: {:?}",
                ty.to_token_stream().to_string()
            );
        }
    }
    res
}

fn ident_vec_comma_separated(
    vec: Vec<(&proc_macro2::Ident, &syn::Type)>,
) -> proc_macro2::TokenStream {
    let tokens = vec.iter().map(|(ident, ty)| {
        quote! { #ident: #ident.into() }
    });
    quote! { #(#tokens),* }
}
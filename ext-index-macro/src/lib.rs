use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, FieldsNamed, LitStr, Token};

const TANTIVY_EXT_TYPES: [&str; 10] = [
    "Tokenized",
    "Str",
    "FastStr",
    "U64",
    "FastU64",
    "F64",
    "FastF64",
    "U64",
    "Date",
    "Score",
];

#[proc_macro_derive(TantivySearchIndex, attributes(tantivy_ext))]
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
    let fields: Vec<syn::Ident> = extract_struct_fields_that_arent_score(&struct_fields);

    let ext_types = get_tantivy_ext_types(&struct_fields);
    let ext_types_tokens = ident_vec_comma_separated(ext_types);

    let mut schema_lines = Vec::new();
    let mut field_fns = Vec::new();
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
                let key = quote! {&self.#field_name.tantivy_val()};
                primary_key_line = Some(field_as_term(field_type, &field_name, key));
            }
        }

        if let Some(schema_field) = schema_field_from_type(field_type, &field_name_str) {
            schema_lines.push(schema_field);
            field_fns.push(create_field_fn(&field_name, field_type));
            as_doc_lines.push(quote! {
                model_fields.#field_name => self.#field_name.tantivy_val(),
            });
        }
        from_doc_lines.push(field_from_doc(field_type, &field_name));
    }
    let model_fields_struct = create_model_fields_struct(struct_name, &fields);
    let model_fields_fn = create_model_fields_fn(struct_name, &fields, &schema_lines);

    let primary_key_impl = primary_key_line.unwrap_or_else(|| {
        panic!(
            "Primary key not specified. Consider annotating a field with `#[\"tantivy_ext(primary_key)\"]`"
        )
    });

    let expanded = quote! {
        use tantivy_ext::ext_type_trait::ExtType;
        use tantivy_ext::Field;
        use tantivy_ext::Index;
        use tantivy::doc;

        #model_fields_struct

        impl ::tantivy_ext::Index for #struct_name {
            fn schema() -> &'static tantivy::schema::Schema {
                &Self::get_model_fields().__schema
            }

            fn get_primary_key(&self) -> tantivy::Term {
                #primary_key_impl
                term
            }

            fn as_document(&self) -> tantivy::TantivyDocument {
                let model_fields = Self::get_model_fields();
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

            fn index_builder(path: std::path::PathBuf) -> ::tantivy_ext::index::index_builder::SearchIndexBuilder<Self>
            where
                Self: std::marker::Sized,
            {
                ::tantivy_ext::index::index_builder::SearchIndexBuilder::new(path)
            }
        }
        impl #struct_name{
            #(#field_fns)*

            #model_fields_fn
        }
    };

    TokenStream::from(expanded)
}

/// Creates this:
/// ```rust
/// struct MyModelFields {
/// name: tantivy::schema::Field,
/// path: tantivy::schema::Field,
/// date: tantivy::schema::Field,
/// popularity: tantivy::schema::Field,
/// }
/// ```
fn create_model_fields_struct(
    struct_name: &syn::Ident,
    fields: &[syn::Ident],
) -> proc_macro2::TokenStream {
    let model_struct_name = proc_macro2::Ident::new(
        &format!("__{}ModelFields", struct_name),
        proc_macro2::Span::call_site(),
    );
    let model_fields: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|field| {
            quote! {
                #field: ::tantivy::schema::Field,
            }
        })
        .collect();
    quote! {
        struct #model_struct_name {
            __schema: ::tantivy::schema::Schema,
            #(#model_fields)*
        }
    }
}

/// Creates this:
/// ```rust
/// fn fields() -> &'static MyModelFields {
///static FIELDS: once_cell::sync::Lazy<MyModelFields> =
///once_cell::sync::Lazy::new(|| {
/// let schema = Self::schema();
/// MyModelFields {
///     name: schema.get_field("name").unwrap(),
///     path: schema.get_field("path").unwrap(),
///      date: schema.get_field("date").unwrap(),
///    popularity: schema.get_field("popularity").unwrap(),
///   }
///});
///&FIELDS
///}
/// ```
fn create_model_fields_fn(
    struct_name: &syn::Ident,
    fields: &[syn::Ident],
    schema_lines: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let model_struct_name = proc_macro2::Ident::new(
        &format!("__{}ModelFields", struct_name),
        proc_macro2::Span::call_site(),
    );
    let schema = quote! {
        {
        let mut schema_builder = tantivy::schema::Schema::builder();
        #(#schema_lines)*
        schema_builder.build()
        }
    };
    let model_fields_assignment: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|field| {
            quote! {
                #field: schema.get_field(stringify!(#field)).unwrap(),
            }
        })
        .collect();
    quote! {
        fn get_model_fields() -> &'static #model_struct_name {
            static FIELDS: ::tantivy_ext::re_exports::once_cell::sync::Lazy<#model_struct_name> =
            ::tantivy_ext::re_exports::once_cell::sync::Lazy::new(||{
                let schema = #schema;
                let schema_clone = schema.clone();
                #model_struct_name {
                    __schema:schema_clone,
                    #(#model_fields_assignment)*
                }
            });
            &FIELDS
        }
    }
}

/// Used for getting the primary key term
fn field_as_term(
    ty: &syn::Type,
    field_name: &syn::Ident,
    key: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_str = &segment.ident.to_string();
            match type_str.as_str() {
                "Tokenized" | "Str" | "FastStr" => {
                    quote! {
                        let term = tantivy::Term::from_field_text(
                            Self::get_model_fields().#field_name,
                            #key,
                        );
                    }
                }
                _ => panic!("Unsupported primary key type: {}. Primary key must be a `Tokenized`, `Str`, or `FastStr` type.", type_str),
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
                "Tokenized" | "Str" | "FastStr" => {
                    quote! {
                        let #field_name = ::tantivy_ext::field_extractor::field_as_string(schema, &doc,stringify!(#field_name)).unwrap();
                    }
                }
                "FastU64" | "U64" => {
                    quote! {
                        let #field_name = ::tantivy_ext::field_extractor::field_as_u64(schema, &doc,stringify!(#field_name)).unwrap();
                    }
                }
                "FastF64" | "F64" => {
                    quote! {
                        let #field_name = ::tantivy_ext::field_extractor::field_as_f64(schema, &doc,stringify!(#field_name)).unwrap();
                    }
                }
                "Date" => {
                    quote! {
                        let #field_name = ::tantivy_ext::field_extractor::field_as_date(schema, &doc,stringify!(#field_name)).unwrap();
                    }
                }
                "Score" => {
                    quote! {
                        let #field_name = score;
                    }
                }
                _ => panic!("Unsupported field type: {}", type_str),
            }
        } else {
            panic!("Invalid type path");
        }
    } else {
        panic!(
            "Unsupported field type: {:?}",
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
                "Tokenized" => Some(
                    quote! { schema_builder.add_text_field(#field_name, tantivy::schema::TEXT | tantivy::schema::STORED); },
                ),
                "Str" => Some(
                    quote! { schema_builder.add_text_field(#field_name, tantivy::schema::STRING | tantivy::schema::STORED); },
                ),
                "FastStr" => Some(
                    quote! { schema_builder.add_text_field(#field_name, tantivy::schema::FAST | tantivy::schema::STRING | tantivy::schema::STORED); },
                ),
                "U64" | "FastU64" => Some(
                    quote! { schema_builder.add_u64_field(#field_name, tantivy::schema::FAST | tantivy::schema::STORED); },
                ),
                "F64" | "FastF64" => Some(
                    quote! { schema_builder.add_f64_field(#field_name, tantivy::schema::FAST | tantivy::schema::STORED); },
                ),
                "Date" => Some(
                    quote! { schema_builder.add_date_field(#field_name, tantivy::schema::INDEXED | tantivy::schema::STORED); },
                ),
                _ => None, // Unknown field. Don't include it in the schema
            }
        } else {
            panic!("Invalid type path");
        }
    } else {
        panic!(
            "Unsupported field type: {:?}",
            ty.to_token_stream().to_string()
        );
    }
}

fn syn_type_to_ext_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    syn_type_to_ext_type_fallible(ty).unwrap()
}

fn syn_type_to_ext_type_fallible(ty: &syn::Type) -> Result<proc_macro2::TokenStream,String> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // Get the last path segment. Example: `fields::FastU64` -> `FastU64`
            let type_str = &segment.ident.to_string();
            return match type_str.as_str() {
                "FastStr" | "Str" | "Tokenized" => Ok(quote! {::tantivy_ext::ext_type::ExtText}),
                "U64" | "FastU64" => Ok(quote! {::tantivy_ext::ext_type::ExtU64}),
                "F64" | "FastF64" => Ok(quote! {::tantivy_ext::ext_type::ExtF64}),
                "Date" => Ok(quote! {::tantivy_ext::ext_type::ExtDate}),
                _ => Err(format!("Unknown EXT field: {}", type_str)),
            };
        } else {
            return Err("Invalid type path".to_string());
        }
    }
    Err(format!(
        "Unsupported field type: {:?}",
        ty.to_token_stream().to_string())
    )
}

fn create_field_fn(field_name: &syn::Ident, field_type: &syn::Type) -> proc_macro2::TokenStream {
    let field_fn_name = proc_macro2::Ident::new(
        &format!("{}_field", field_name),
        proc_macro2::Span::call_site(),
    );
    // This will be used as the generic
    let ext_type = syn_type_to_ext_type(field_type);
    quote! {
        pub fn #field_fn_name() -> ::tantivy_ext::ext_field::ExtField::<#ext_type>{
            ::tantivy_ext::ext_field::ExtField::new(
                stringify!(#field_name).to_string(),
                Self::get_model_fields().#field_name
            )
        }
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
                "Unsupported field type: {:?}",
                ty.to_token_stream().to_string()
            );
        }
    }
    res
}

fn ident_vec_comma_separated(
    vec: Vec<(&proc_macro2::Ident, &syn::Type)>,
) -> proc_macro2::TokenStream {
    let tokens = vec.iter().map(|(ident, _ty)| {
        quote! { #ident: #ident.into() }
    });
    quote! { #(#tokens),* }
}

/// Returns the identifiers to those said fields
fn extract_struct_fields_that_arent_score(fields:&FieldsNamed)->Vec<syn::Ident>{
    let mut idents = Vec::new();
    for field in fields.named.iter(){
        if let Ok(_) =  syn_type_to_ext_type_fallible(&field.ty){
            idents.push(field.ident.clone().unwrap())
        }
    }
    idents
}
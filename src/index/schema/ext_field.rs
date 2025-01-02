use tantivy::schema::Field;

/// Wrapper around the `tantivy::schema::Field` struct.
/// 
/// Can be coerced into a `tantivy::schema::Field` to get the real field, or a `String` in order to get the name of the field as it is represented in the schema.
pub struct ExtField {
    field: Field,
    field_name: String,
}

impl ExtField {
    pub fn new(field: Field, field_name: String) -> Self {
        Self { field, field_name }
    }
}

impl From<ExtField> for Field{
    fn from(value: ExtField) -> Self {
        value.field
    }
}


impl From<ExtField> for String{
    fn from(value: ExtField) -> Self {
        value.field_name
    }
}
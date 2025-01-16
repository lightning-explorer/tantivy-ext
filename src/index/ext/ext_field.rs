use tantivy::schema::Field;

use super::ext_type_trait::ExtType;

/// Wrapper around the `tantivy::schema::Field` struct.
///
/// Can be coerced into a `tantivy::schema::Field` to get the real field, or a `String` in order to get the name of the field as it is represented in the schema.
pub struct ExtField<T>
where
    T: ExtType,
{
    ext_type: T,
}

impl<T> ExtField<T>
where
    T: ExtType,
{
    pub fn new(field_name: String, field: Field) -> Self {
        Self {
            ext_type: T::new_from_field(field, field_name),
        }
    }
    pub fn term(&self, input: T::Target) -> tantivy::Term {
        self.ext_type.term(input)
    }
}

impl<T> From<ExtField<T>> for Field
where
    T: ExtType,
{
    fn from(value: ExtField<T>) -> Self {
        *value
            .ext_type
            .field()
    }
}

impl<T> From<ExtField<T>> for String
where
    T: ExtType,
{
    fn from(value: ExtField<T>) -> Self {
        value.ext_type.name()
    }
}

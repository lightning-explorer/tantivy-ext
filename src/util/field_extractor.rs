use tantivy::{schema::{OwnedValue, Schema}, Document, TantivyDocument};

pub fn field_as_string(schema: &Schema, doc: &TantivyDocument, field_name: &str) -> Option<String> {
    for (field, value) in doc.iter_fields_and_values() {
        if schema.get_field_name(field) == field_name {
            if let OwnedValue::Str(text) = value {
                return Some(text.to_string());
            }
        }
    }
    None
}

pub fn field_as_date(schema: &Schema, doc: &TantivyDocument, field_name: &str) -> Option<tantivy::DateTime> {
    for (field, value) in doc.iter_fields_and_values() {
        if schema.get_field_name(field) == field_name {
            if let OwnedValue::Date(date) = value {
                return Some(*date)
            }
        }
    }
    None
}

pub fn field_as_u64(schema: &Schema, doc: &TantivyDocument, field_name: &str)->Option<u64>{
    for (field, value) in doc.iter_fields_and_values() {
        if schema.get_field_name(field) == field_name {
            if let OwnedValue::U64(val) = value {
                return Some(*val)
            }
        }
    }
    None
}

pub fn field_as_f64(schema: &Schema, doc: &TantivyDocument, field_name: &str)->Option<f64>{
    for (field, value) in doc.iter_fields_and_values() {
        if schema.get_field_name(field) == field_name {
            if let OwnedValue::F64(val) = value {
                return Some(*val)
            }
        }
    }
    None
}
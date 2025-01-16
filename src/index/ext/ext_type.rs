use tantivy::schema::Field;

use super::ext_type_trait::ExtType;

pub struct ExtText(Field, String);
impl ExtType for ExtText{
    type Target = String;
    fn new_from_field(schema:Field, field_name:String)->Self{
        Self(schema, field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn field(&self)->&Field {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field =  *self.field();
        tantivy::Term::from_field_text(field, &input)
    }
}

pub struct ExtF64(Field, String);
impl ExtType for ExtF64{
    type Target = f64;
    fn new_from_field(schema:Field, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn field(&self)->&Field {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field =  *self.field();
        tantivy::Term::from_field_f64(field, input)
    }
}

pub struct ExtU64(Field, String);
impl ExtType for ExtU64{
    type Target = u64;
    fn new_from_field(schema:Field, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn field(&self)->&Field {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = *self.field();
        tantivy::Term::from_field_u64(field, input)
    }
}

pub struct ExtDate(Field, String);
impl ExtType for ExtDate{
    type Target = tantivy::DateTime;
    fn new_from_field(schema:Field, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn field(&self)->&Field {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = *self.field();
        tantivy::Term::from_field_date(field, input)
    }
}
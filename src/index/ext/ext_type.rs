use tantivy::schema::Schema;

use super::ext_type_trait::ExtType;

pub struct ExtText(Schema, String);
impl ExtType for ExtText{
    type Target = String;
    fn new_from_schema(schema:Schema, field_name:String)->Self{
        Self(schema, field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn schema(&self)->&Schema {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = self.schema().get_field(&self.1).unwrap();
        tantivy::Term::from_field_text(field, &input)
    }
}

pub struct ExtF64(Schema, String);
impl ExtType for ExtF64{
    type Target = f64;
    fn new_from_schema(schema:Schema, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn schema(&self)->&Schema {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = self.schema().get_field(&self.1).unwrap();
        tantivy::Term::from_field_f64(field, input)
    }
}

pub struct ExtU64(Schema, String);
impl ExtType for ExtU64{
    type Target = u64;
    fn new_from_schema(schema:Schema, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn schema(&self)->&Schema {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = self.schema().get_field(&self.1).unwrap();
        tantivy::Term::from_field_u64(field, input)
    }
}

pub struct ExtDate(Schema, String);
impl ExtType for ExtDate{
    type Target = tantivy::DateTime;
    fn new_from_schema(schema:Schema, field_name:String)->Self{
        Self(schema,field_name)
    }
    fn name(&self)->String {
        self.1.clone()
    }
    fn schema(&self)->&Schema {
        &self.0
    }
    fn term(&self,input:Self::Target)->tantivy::Term {
        let field = self.schema().get_field(&self.1).unwrap();
        tantivy::Term::from_field_date(field, input)
    }
}
use tantivy::schema::Field;

pub trait ExtType{
    type Target;
    fn new_from_field(field:Field, field_name:String)->Self;
    fn field(&self)->&Field;
    /** The name of this field */
    fn name(&self)->String;
    fn term(&self,input:Self::Target)->tantivy::Term;
}
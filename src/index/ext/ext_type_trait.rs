use tantivy::schema::Schema;

pub trait ExtType{
    type Target;
    fn new_from_schema(schema:Schema, field_name:String)->Self;
    fn schema(&self)->&Schema;
    /** The name of this field */
    fn name(&self)->String;
    fn term(&self,input:Self::Target)->tantivy::Term;
}
use lazy_static::lazy_static;

use meta_store::Field;

lazy_static! {
    pub static ref A_IDENTIFIER: Field = "0".into(); // identifier
    pub static ref A_ATTRIBUTE_VALUE_TYPE: Field = "1".into(); // Attribute.value-type
    pub static ref V_STRING: Field = "2".into(); // String
    pub static ref V_REFERENCE: Field = "3".into(); // Reference
    pub static ref A_COMMENT: Field = "4".into(); // comment
    pub static ref A_TYPE: Field = "5".into(); // type
    pub static ref T_TYPE: Field = "6".into(); // Type
    pub static ref T_ATTRIBUTE: Field = "7".into(); // Attribute
    pub static ref A_VALUE_TYPE: Field = "8".into(); // ValueType
    pub static ref V_NATURAL_NUMBER: Field = "9".into(); // NaturalNumber
    pub static ref A_ATTRIBUTE_REFERENCE_TYPE: Field = "10".into(); // Attribute.reference-type
    pub static ref V_INTEGER_NUMBER: Field = "11".into(); // IntegerNumber
    pub static ref T_LANGUAGE: Field = "12".into(); // Language
    pub static ref A_LANGUAGE_ENTITY: Field = "13".into(); // Language.entity
    pub static ref A_TYPE_ATTRIBUTE: Field = "15".into(); // Type.attribute
    pub static ref A_AFTER: Field = "16".into(); // after
    pub static ref META_CORE: Field = "14".into(); // meta.core
}

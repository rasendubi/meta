use lazy_static::lazy_static;

use meta_store::Field;

lazy_static! {
    pub static ref ENTRY_POINT: Field = Field::from("ckgrnb2q20000xamazg71jcf6");
    pub static ref ENTRY_POINT_EXPR: Field = Field::from("ckgrnjxj30006xamalz6xvuk7");
    pub static ref NUMBER_LITERAL: Field = "ckgkz9xrn0009q2ma3hyzyejp".into();
    pub static ref NUMBER_LITERAL_VALUE: Field = "ckgkzbdt1000fq2maaedmj0rd".into();
    pub static ref STRING_LITERAL: Field = "ckgkz6klf0000q2mas3dh1ms1".into();
    pub static ref STRING_LITERAL_VALUE: Field = "ckgkz7deb0004q2maroxbccv8".into();
    pub static ref FUNCTION: Field = "ckgvae1350000whmaqi356557".into();
    pub static ref FUNCTION_BODY: Field = "ckgvag4va0004whmadyh1qnnv".into();
    pub static ref FUNCTION_PARAMETER: Field = "ckgvahph5000bwhmaias0bwf7".into();
    pub static ref PARAMETER: Field = "ckgz410en000d9hmazxmz6hqy".into();
    pub static ref PARAMETER_IDENTIFIER: Field = "ckgz42xkx000s9hma2njbx3i7".into();
    pub static ref APPLICATION: Field = "ckgxipqk50000c7mawkssuook".into();
    pub static ref APPLICATION_FN: Field = "ckgxiq1ot0004c7maalcx609z".into();
    pub static ref APPLICATION_ARGUMENT: Field = "ckgxiqlw50009c7mask5ery0g".into();
    pub static ref BLOCK: Field = "ckgz33mrp00005omaq226vzth".into();
    pub static ref BLOCK_STATEMENT: Field = "ckgz33vst00045omakt15dloc".into();
    pub static ref IDENTIFIER: Field = "ckgz4197i000h9hmazilan75h".into();
    pub static ref IDENTIFIER_IDENTIFIER: Field = "ckgz41sua000l9hma691bmbeh".into();
    pub static ref BINDING: Field = "ckgvali04000hwhmaw93ym25w".into();
    pub static ref BINDING_IDENTIFIER: Field = "ckgvaluy0000lwhmai73hadxb".into();
    pub static ref BINDING_VALUE: Field = "ckgvamn7n000rwhmaz95psjz9".into();
}

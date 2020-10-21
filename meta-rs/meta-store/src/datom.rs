use std::{fmt::Debug, result::Result};

use cuid::cuid;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use string_cache::{Atom, DefaultAtom};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub struct Field(DefaultAtom);

impl Field {
    pub fn new_id() -> Self {
        Self(cuid().expect("cuid failure").into())
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Field").field(&self.as_ref()).finish()
    }
}

impl AsRef<str> for Field {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for Field {
    #[inline]
    fn from(s: String) -> Self {
        Field(Atom::from(s))
    }
}

impl<'a> From<&'a str> for Field {
    #[inline]
    fn from(s: &'a str) -> Self {
        Field(Atom::from(s))
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Datom {
    pub id: Field,
    pub entity: Field,
    pub attribute: Field,
    pub value: Field,
}

impl Debug for Datom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Datom")
            .field(&self.id.as_ref())
            .field(&self.entity.as_ref())
            .field(&self.attribute.as_ref())
            .field(&self.value.as_ref())
            .finish()
    }
}

impl<'a> From<(&'a str, &'a str, &'a str, &'a str)> for Datom {
    fn from(x: (&'a str, &'a str, &'a str, &'a str)) -> Self {
        Datom::new(x.0.into(), x.1.into(), x.2.into(), x.3.into())
    }
}

impl Datom {
    #[inline]
    pub fn new(id: Field, entity: Field, attribute: Field, value: Field) -> Datom {
        Datom {
            id,
            entity,
            attribute,
            value,
        }
    }

    pub fn eav(entity: Field, attribute: Field, value: Field) -> Datom {
        let id = cuid().expect("cuid generation error").into();
        Datom {
            id,
            entity,
            attribute,
            value,
        }
    }
}

impl Serialize for Datom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.entity)?;
        seq.serialize_element(&self.attribute)?;
        seq.serialize_element(&self.value)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Datom {
    fn deserialize<D>(deserializer: D) -> Result<Datom, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<String> = Deserialize::deserialize(deserializer)?;

        match &v.as_slice() {
            [e, a, v] => Ok(Datom::eav(
                e.as_str().into(),
                a.as_str().into(),
                v.as_str().into(),
            )),
            [i, e, a, v] => Ok(Datom::new(
                i.as_str().into(),
                e.as_str().into(),
                a.as_str().into(),
                v.as_str().into(),
            )),
            _ => Err(serde::de::Error::invalid_length(v.len(), &"3 or 4")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn datom_deserialize_array() {
        assert_eq!(
            serde_json::from_str::<Datom>(r#"["0", "1", "2", "3"]"#).unwrap(),
            ("0", "1", "2", "3").into()
        );
    }

    #[test]
    fn datom_deserialize_three_tuple() {
        let r = serde_json::from_str::<Datom>(r#"["1", "2", "3"]"#).unwrap();

        assert_eq!(r.entity, "1".into());
        assert_eq!(r.attribute, "2".into());
        assert_eq!(r.value, "3".into());
    }

    #[test]
    fn datom_serialize_deserialize() {
        let x = ("0", "1", "2", "3").into();
        assert_eq!(
            serde_json::from_str::<Datom>(&serde_json::to_string(&x).unwrap()).unwrap(),
            x
        );
    }

    #[test]
    fn datom_deserialize_escape() {
        assert_eq!(
            serde_json::from_str::<Datom>(r#"["0", "1", "2", "\"hello\""]"#).unwrap(),
            ("0", "1", "2", "\"hello\"").into()
        );
    }
}

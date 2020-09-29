use cuid::cuid;
use string_cache::{Atom, DefaultAtom};

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Field(DefaultAtom);

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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Datom {
    pub id: Field,
    pub entity: Field,
    pub attribute: Field,
    pub value: Field,
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
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
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
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Datom, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<&str> = Deserialize::deserialize(deserializer)?;

        match v.len() {
            3 => unsafe {
                Ok(Datom::eav(
                    (*v.get_unchecked(0)).into(),
                    (*v.get_unchecked(1)).into(),
                    (*v.get_unchecked(2)).into(),
                ))
            },
            4 => unsafe {
                Ok(Datom::new(
                    (*v.get_unchecked(0)).into(),
                    (*v.get_unchecked(1)).into(),
                    (*v.get_unchecked(2)).into(),
                    (*v.get_unchecked(3)).into(),
                ))
            },
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
}

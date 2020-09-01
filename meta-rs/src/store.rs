use std::collections::{HashMap, HashSet};

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};

use string_cache::{Atom, DefaultAtom};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub struct Field(DefaultAtom);

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

#[derive(Debug, PartialEq, Eq)]
pub struct Datom {
    entity: Field,
    attribute: Field,
    value: Field,
}

#[derive(Debug)]
struct Index(HashMap<Field, HashMap<Field, HashSet<Field>>>);

#[derive(Debug)]
pub struct MetaStore {
    eav: Index,
    aev: Index,
    ave: Index,
}

impl MetaStore {
    pub fn new() -> MetaStore {
        MetaStore {
            eav: Index::new(),
            aev: Index::new(),
            ave: Index::new(),
        }
    }

    pub fn from_reader<R>(r: R) -> Result<MetaStore>
    where
        R: std::io::BufRead,
    {
        let mut store = MetaStore::new();

        for line in r.lines() {
            let line = line?;
            let mut datoms = serde_json::Deserializer::from_str(&line).into_iter::<Datom>();
            if let Some(Ok(datom)) = datoms.next() {
                store.add_datom(&datom);
            }
        }

        Ok(store)
    }

    pub fn from_str(s: &str) -> Result<MetaStore> {
        MetaStore::from_reader(std::io::Cursor::new(s))
    }

    pub fn add_datom(&mut self, datom: &Datom) {
        let Datom {
            entity,
            attribute,
            value,
        } = datom;
        self.eav
            .add(entity.clone(), attribute.clone(), value.clone());
        self.aev
            .add(attribute.clone(), entity.clone(), value.clone());
        self.ave
            .add(attribute.clone(), value.clone(), entity.clone());
    }

    #[inline]
    pub fn eav1(&self, e: &Field) -> Option<&HashMap<Field, HashSet<Field>>> {
        self.eav.get(e)
    }

    #[inline]
    pub fn eav2(&self, e: &Field, a: &Field) -> Option<&HashSet<Field>> {
        self.eav.get(e)?.get(a)
    }

    #[inline]
    pub fn aev1(&self, a: &Field) -> Option<&HashMap<Field, HashSet<Field>>> {
        self.aev.get(a)
    }

    #[inline]
    pub fn aev2(&self, a: &Field, e: &Field) -> Option<&HashSet<Field>> {
        self.aev.get(a)?.get(e)
    }

    #[inline]
    pub fn ave1(&self, a: &Field) -> Option<&HashMap<Field, HashSet<Field>>> {
        self.ave.get(a)
    }

    #[inline]
    pub fn ave2(&self, a: &Field, v: &Field) -> Option<&HashSet<Field>> {
        self.ave.get(a)?.get(v)
    }

    #[inline]
    pub fn values(&self, e: &Field, a: &Field) -> Option<&HashSet<Field>> {
        self.eav2(e, a)
    }

    pub fn value(&self, e: &Field, a: &Field) -> Option<&Field> {
        self.values(e, a)?.iter().next()
    }
}

impl Index {
    pub fn new() -> Index {
        Index(HashMap::new())
    }

    pub fn add(&mut self, x: Field, y: Field, z: Field) {
        let yzs = self.0.entry(x).or_insert_with(HashMap::new);
        let zs = yzs.entry(y).or_insert_with(HashSet::new);
        zs.insert(z);
    }

    #[inline]
    pub fn get(&self, x: &Field) -> Option<&HashMap<Field, HashSet<Field>>> {
        self.0.get(x)
    }
}

impl Datom {
    #[inline]
    pub fn new(entity: Field, attribute: Field, value: Field) -> Datom {
        Datom {
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
        let (entity, attribute, value): (String, String, String) =
            Deserialize::deserialize(deserializer)?;
        Ok(Datom::new(
            Field::from(entity),
            Field::from(attribute),
            Field::from(value),
        ))
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(::std::io::Error),
    JsonError(serde_json::Error),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::IoError(ref e) => e.fmt(f),
            Error::JsonError(ref e) => e.fmt(f),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::{hashmap, hashset};
    use serde_json;

    static TEST: &str = r#"
        ["0", "0", "identifier"]
        ["0", "1", "2"]
        ["1", "1", "3"]
        ["1", "0", "Attribute.value-type"]
        ["2", "0", "String"]
        ["3", "0", "Reference"]
        ["4", "0", "comment"]
        ["0", "4", "Unique identifier of element"]
        ["0", "4", "Additional comment"]
    "#;

    #[test]
    fn datom_deserialize_array() {
        let x = Datom::new(Field::from("1"), Field::from("2"), Field::from("3"));
        assert_eq!(
            serde_json::from_str::<Datom>(r#"["1", "2", "3"]"#).unwrap(),
            x
        );
    }

    #[test]
    fn datom_serialize_deserialize() {
        let x = Datom::new(Field::from("1"), Field::from("2"), Field::from("3"));
        assert_eq!(
            serde_json::from_str::<Datom>(&serde_json::to_string(&x).unwrap()).unwrap(),
            x
        );
    }

    #[test]
    fn store_from_buf() {
        let _store = MetaStore::from_reader(std::io::Cursor::new(TEST)).unwrap();
    }

    #[test]
    fn store_parse_trailing_comment() {
        let store = MetaStore::from_str(r#"["0", "0", "identifier"] ;; trailing comment"#).unwrap();
        assert_eq!(
            Some(&hashset! {Field::from("identifier")}),
            store.eav2(&Field::from("0"), &Field::from("0"))
        );
    }

    #[test]
    fn get_by_entity_attribute() {
        let store = MetaStore::from_str(TEST).unwrap();

        assert_eq!(
            Some(&hashset! {Field::from("identifier")}),
            store.eav2(&Field::from("0"), &Field::from("0"))
        );
    }

    #[test]
    fn get_entity() {
        let store = MetaStore::from_str(TEST).unwrap();

        assert_eq!(
            Some(&hashmap! {
                Field::from("0") => hashset!{Field::from("identifier")},
                Field::from("1") => hashset!{Field::from("2")},
                Field::from("4") => hashset!{Field::from("Unique identifier of element"),
                                             Field::from("Additional comment")},
            }),
            store.eav1(&Field::from("0"))
        );
    }
}

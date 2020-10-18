mod datom;

use im::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

pub use crate::datom::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Index(HashMap<Field, HashMap<Field, HashSet<Datom>>>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Store {
    atoms: HashMap</* id: */ Field, Datom>,
    eav: Index,
    aev: Index,
    ave: Index,
}

impl Store {
    pub fn new() -> Store {
        Store {
            atoms: HashMap::new(),
            eav: Index::new(),
            aev: Index::new(),
            ave: Index::new(),
        }
    }

    pub fn from_reader<R>(r: R) -> Result<Store>
    where
        R: std::io::BufRead,
    {
        let mut store = Store::new();

        for line in r.lines() {
            let line = line?;
            let mut datoms = serde_json::Deserializer::from_str(&line).into_iter::<Datom>();
            if let Some(Ok(datom)) = datoms.next() {
                store.add_datom(&datom);
            }
        }

        Ok(store)
    }

    pub fn add_datom(&mut self, datom: &Datom) {
        let Datom {
            id,
            entity,
            attribute,
            value,
        } = datom;
        self.atoms.insert(id.clone(), datom.clone());
        self.eav
            .add(entity.clone(), attribute.clone(), datom.clone());
        self.aev
            .add(attribute.clone(), entity.clone(), datom.clone());
        self.ave
            .add(attribute.clone(), value.clone(), datom.clone());
    }

    pub fn remove_datom(&mut self, datom: &Datom) {
        let Datom {
            id,
            entity,
            attribute,
            value,
        } = datom;
        self.atoms.remove(id);
        self.eav.remove(entity.clone(), attribute.clone(), &datom);
        self.aev.remove(attribute.clone(), entity.clone(), &datom);
        self.ave.remove(attribute.clone(), value.clone(), &datom);
    }

    pub fn atoms(&self) -> &HashMap<Field, Datom> {
        &self.atoms
    }

    #[inline]
    pub fn eav1(&self, e: &Field) -> Option<&HashMap<Field, HashSet<Datom>>> {
        self.eav.get(e)
    }

    #[inline]
    pub fn eav2(&self, e: &Field, a: &Field) -> Option<&HashSet<Datom>> {
        self.eav.get(e)?.get(a)
    }

    #[inline]
    pub fn aev1(&self, a: &Field) -> Option<&HashMap<Field, HashSet<Datom>>> {
        self.aev.get(a)
    }

    #[inline]
    pub fn aev2(&self, a: &Field, e: &Field) -> Option<&HashSet<Datom>> {
        self.aev.get(a)?.get(e)
    }

    #[inline]
    pub fn ave1(&self, a: &Field) -> Option<&HashMap<Field, HashSet<Datom>>> {
        self.ave.get(a)
    }

    #[inline]
    pub fn ave2(&self, a: &Field, v: &Field) -> Option<&HashSet<Datom>> {
        self.ave.get(a)?.get(v)
    }

    pub fn entities(&self) -> HashSet<&Field> {
        self.eav.0.keys().collect()
    }

    #[inline]
    pub fn values(&self, e: &Field, a: &Field) -> Option<&HashSet<Datom>> {
        self.eav2(e, a)
    }

    pub fn value(&self, e: &Field, a: &Field) -> Option<&Datom> {
        self.values(e, a)?.iter().next()
    }
}

impl Serialize for Store {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut datoms = self.atoms.values().collect::<Vec<_>>();
        datoms.sort();
        serializer.serialize_newtype_struct("Store", &datoms)
    }
}

impl<'de> Deserialize<'de> for Store {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let datoms = Vec::<Datom>::deserialize(deserializer)?;

        let mut store = Self::new();
        for datom in datoms {
            store.add_datom(&datom);
        }

        Ok(store)
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for Store {
    type Err = Error;

    fn from_str(s: &str) -> Result<Store> {
        Store::from_reader(std::io::Cursor::new(s))
    }
}

impl Index {
    pub fn new() -> Index {
        Index(HashMap::new())
    }

    pub fn add(&mut self, x: Field, y: Field, datom: Datom) {
        let yzs = self.0.entry(x).or_insert_with(HashMap::new);
        let zs = yzs.entry(y).or_insert_with(HashSet::new);
        zs.insert(datom);
    }

    pub fn remove(&mut self, x: Field, y: Field, datom: &Datom) {
        fn non_empty_map<K, V>(m: HashMap<K, V>) -> Option<HashMap<K, V>> {
            if m.is_empty() {
                None
            } else {
                Some(m)
            }
        }
        fn non_empty_set<V>(s: HashSet<V>) -> Option<HashSet<V>> {
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }

        let update_zs =
            |mzs: Option<HashSet<Datom>>| mzs.map(|zs| zs.without(&datom)).and_then(non_empty_set);
        let update_yzs = |myzs: Option<HashMap<Field, HashSet<Datom>>>| {
            myzs.map(|yzs| yzs.alter(update_zs, y))
                .and_then(non_empty_map)
        };
        self.0 = self.0.alter(update_yzs, x);
    }

    #[inline]
    pub fn get(&self, x: &Field) -> Option<&HashMap<Field, HashSet<Datom>>> {
        self.0.get(x)
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
    use im::{hashmap, hashset};
    use std::str::FromStr;

    static TEST: &str = r#"
        ["-1", "0", "0", "identifier"]
        ["-2", "0", "1", "2"]
        ["-3", "1", "1", "3"]
        ["-4", "1", "0", "Attribute.value-type"]
        ["-5", "2", "0", "String"]
        ["-6", "3", "0", "Reference"]
        ["-7", "4", "0", "comment"]
        ["-8", "0", "4", "Unique identifier of element"]
        ["-9", "0", "4", "Additional comment"]
    "#;

    #[test]
    fn store_from_buf() {
        let _store = Store::from_reader(std::io::Cursor::new(TEST)).unwrap();
    }

    #[test]
    fn store_parse_trailing_comment() {
        let store =
            Store::from_str(r#"["2", "0", "0", "identifier"] ;; trailing comment"#).unwrap();
        assert_eq!(
            Some(&hashset! {("2", "0", "0", "identifier").into()}),
            store.eav2(&Field::from("0"), &Field::from("0"))
        );
    }

    #[test]
    fn get_by_entity_attribute() {
        let store = Store::from_str(TEST).unwrap();

        assert_eq!(
            Some(&hashset! {("-1", "0", "0", "identifier").into()}),
            store.eav2(&Field::from("0"), &Field::from("0"))
        );
    }

    #[test]
    fn get_entity() {
        let store = Store::from_str(TEST).unwrap();

        assert_eq!(
            Some(&hashmap! {
                Field::from("0") => hashset!{("-1", "0", "0", "identifier").into()},
                Field::from("1") => hashset!{("-2", "0", "1", "2").into()},
                Field::from("4") => hashset!{("-8", "0", "4", "Unique identifier of element").into(),
                                             ("-9", "0", "4", "Additional comment").into()},
            }),
            store.eav1(&Field::from("0"))
        );
    }

    #[test]
    fn remove_datom() {
        let mut store = Store::from_str(TEST).unwrap();
        store.remove_datom(&("-2", "0", "1", "2").into());
        store.remove_datom(&("-9", "0", "4", "Additional comment").into());
        assert_eq!(
            Some(&hashmap! {
                Field::from("0") => hashset!{("-1", "0", "0", "identifier").into()},
                Field::from("4") => hashset!{("-8", "0", "4", "Unique identifier of element").into()},
            }),
            store.eav1(&Field::from("0"))
        );
    }
}

//! Some helpers for the meta.core language.
pub mod ids;

use im::{HashMap, HashSet};
use itertools::Itertools;

use meta_store::{Datom, Field, Store};

use crate::ids::*;

pub struct MetaCore<'a> {
    pub store: &'a Store,
    // TODO: remove hard-code ids and cache them in struct
}

impl<'a> MetaCore<'a> {
    pub fn new(store: &Store) -> MetaCore {
        MetaCore { store }
    }

    pub fn identifier(&self, entity: &Field) -> Option<&Datom> {
        self.store.value(entity, &A_IDENTIFIER)
    }

    pub fn meta_type(&self, entity: &Field) -> Option<&Datom> {
        self.store.value(entity, &A_TYPE)
    }

    pub fn meta_attribute_type(&self, entity: &Field) -> Option<&Datom> {
        self.store.value(entity, &A_ATTRIBUTE_VALUE_TYPE)
    }

    pub fn meta_attribute_reference_type(&self, entity: &Field) -> Option<&HashSet<Datom>> {
        self.store.values(entity, &A_ATTRIBUTE_REFERENCE_TYPE)
    }

    pub fn after(&self, datom: &Datom) -> Option<&Field> {
        self.store.value(&datom.id, &A_AFTER).map(|a| &a.value)
    }

    pub fn of_type(&self, type_: &Field) -> HashSet<Datom> {
        self.store
            .ave2(&A_TYPE, type_)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    /// Order atoms in order determines by `after` attribute. If `after` is not specified, order by
    /// atom id.
    // Believe me or not, it's actually O(n + m*log(m)), where n is the total number of datoms and m
    // is the number of atoms without "after" attribute.
    pub fn order_datoms<I>(&'a self, atoms: I) -> Vec<&'a Datom>
    where
        I: IntoIterator<Item = &'a Datom>,
    {
        let mut no_after = HashSet::new();
        let mut next = HashMap::<&Field, HashSet<&Datom>>::new();
        for x in atoms.into_iter() {
            if let Some(a) = self.after(x) {
                next.entry(a).or_insert_with(HashSet::new).insert(x);
            } else {
                no_after.insert(x);
            }
        }

        // it would be much easier if Rust allowed recursive closures
        fn traverse_atom<'a>(
            x: &'a Datom,
            result: &'_ mut Vec<&'a Datom>,
            next: &HashMap<&'a Field, HashSet<&'a Datom>>,
        ) {
            result.push(x);
            if let Some(next_atoms) = next.get(&x.id) {
                for a in next_atoms.iter() {
                    traverse_atom(a, result, next);
                }
            }
        }

        let mut result = Vec::new();
        for a in no_after.iter().sorted_by_key(|x| &x.id) {
            traverse_atom(a, &mut result, &next);
        }

        result
    }

    pub fn ordered_values(&self, entry: &Field, attr: &Field) -> Vec<&Datom> {
        self.store
            .values(entry, attr)
            .map_or_else(Vec::new, |datoms| self.order_datoms(datoms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use meta_store::{Field, Store};
    use std::str::FromStr;

    static TEST: &'static str = include_str!("../../../core.meta");

    #[test]
    fn test_identifier() {
        let store = Store::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        assert_eq!(
            Some(&Field::from("identifier")),
            core.identifier(&Field::from("0")).map(|x| &x.value)
        );
    }

    #[test]
    fn test_meta_type() {
        let store = Store::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        let attribute = Field::from("7");
        assert_eq!(
            Some(&attribute),
            core.meta_type(&Field::from("0")).map(|x| &x.value)
        );
    }

    #[test]
    fn test_meta_attribute_type() {
        let store = Store::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        let string = Field::from("2");
        assert_eq!(
            Some(&string),
            core.meta_attribute_type(&Field::from("0"))
                .map(|x| &x.value)
        );
    }

    #[test]
    fn test_order_no_after() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| core.order_datoms(x));

        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("11", "0", "1", "3").into(),
                &("12", "0", "1", "4").into(),
            ] as Vec<&Datom>,
            result
        );
    }

    #[test]
    fn test_order_with_after() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
              ["13", "12", "16", "10"]
              ["14", "11", "16", "12"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| core.order_datoms(x));

        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("12", "0", "1", "4").into(),
                &("11", "0", "1", "3").into(),
            ] as Vec<&Datom>,
            result
        );
    }

    #[test]
    #[ignore] // TODO: order silently drops all loops now (a after b, b after a)
    fn test_order_with_after_loop() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
              ["13", "12", "16", "10"]
              ["14", "11", "16", "12"]
              ["15", "10", "16", "11"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| core.order_datoms(x));

        // if loop is detected, prefer starting from the lowest id
        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("12", "0", "1", "4").into(),
                &("11", "0", "1", "3").into(),
            ] as Vec<&Datom>,
            result
        );
    }
}

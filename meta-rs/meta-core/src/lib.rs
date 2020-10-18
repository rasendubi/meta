//! Some helpers for the meta.core language.
use im::HashSet;

use meta_store::{Datom, Field, Store};

pub struct MetaCore<'a> {
    pub store: &'a Store,
    // TODO: remove hard-code ids and cache them in struct
}

impl<'a> MetaCore<'a> {
    pub fn new(store: &Store) -> MetaCore {
        MetaCore { store }
    }

    pub fn identifier(&self, entity: &Field) -> Option<&Datom> {
        let identifier = Field::from("0");
        self.store.value(entity, &identifier)
    }

    pub fn meta_type(&self, entity: &Field) -> Option<&Datom> {
        let type_ = Field::from("5");
        self.store.value(entity, &type_)
    }

    pub fn meta_attribute_type(&self, entity: &Field) -> Option<&Datom> {
        let attribute_type = Field::from("1");
        self.store.value(entity, &attribute_type)
    }

    pub fn meta_attribute_reference_type(&self, entity: &Field) -> Option<&HashSet<Datom>> {
        let attribute_reference_type = "10".into();
        self.store.values(entity, &attribute_reference_type)
    }

    pub fn after(&self, datom: &Datom) -> Option<&Field> {
        let after_id = Field::from("16");
        self.store.value(&datom.id, &after_id).map(|a| &a.value)
    }

    pub fn of_type(&self, type_: &Field) -> HashSet<Datom> {
        let type_id = "5".into();
        self.store
            .ave2(&type_id, type_)
            .cloned()
            .unwrap_or_else(HashSet::new)
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
}

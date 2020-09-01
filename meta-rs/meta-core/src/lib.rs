//! Some helpers for the meta.core language.
use meta_store::{Field, MetaStore};

pub struct MetaCore<'a> {
    store: &'a MetaStore,
    // TODO: remove hard-code ids and cache them in struct
}

impl<'a> MetaCore<'a> {
    pub fn new(store: &MetaStore) -> MetaCore {
        MetaCore { store }
    }

    pub fn identifier(&self, entity: &Field) -> Option<&Field> {
        let identifier = Field::from("0");
        self.store.value(entity, &identifier)
    }

    pub fn meta_type(&self, entity: &Field) -> Option<&Field> {
        let type_ = Field::from("5");
        self.store.value(entity, &type_)
    }

    pub fn meta_attribute_type(&self, entity: &Field) -> Option<&Field> {
        let attribute_type = Field::from("1");
        self.store.value(entity, &attribute_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use meta_store::{Field, MetaStore};
    use std::str::FromStr;

    static TEST: &'static str = include_str!("../../../core.meta");

    #[test]
    fn test_identifier() {
        let store = MetaStore::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        assert_eq!(
            Some(&Field::from("identifier")),
            core.identifier(&Field::from("0"))
        );
    }

    #[test]
    fn test_meta_type() {
        let store = MetaStore::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        let attribute = Field::from("7");
        assert_eq!(Some(&attribute), core.meta_type(&Field::from("0")));
    }

    #[test]
    fn test_meta_attribute_type() {
        let store = MetaStore::from_str(TEST).unwrap();
        let core = MetaCore::new(&store);

        let string = Field::from("2");
        assert_eq!(Some(&string), core.meta_attribute_type(&Field::from("0")));
    }
}

use druid_shell::{HotKey, KeyCode, KeyEvent, SysMods};

use meta_core::ids;
use meta_core::MetaCore;
use meta_store::{Datom, Field, Store};

use crate::editor::Editor;
use crate::key::KeyHandler;
use im::HashSet;
use log::{trace, warn};

#[derive(Debug)]
pub(crate) struct ReorderKeys(pub Datom);

impl KeyHandler for ReorderKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::ArrowUp).matches(key) {
            self.reorder(editor, Direction::Up);
            return true;
        }

        if HotKey::new(SysMods::Cmd, KeyCode::ArrowDown).matches(key) {
            self.reorder(editor, Direction::Down);
            return true;
        }

        false
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Direction {
    Up = -1,
    Down = 1,
}

impl ReorderKeys {
    fn reorder(&self, editor: &mut Editor, direction: Direction) {
        let Datom {
            id,
            entity,
            attribute,
            value: _,
        } = &self.0;

        editor.with_store(|store| {
            let id_other = {
                let core = MetaCore::new(store);
                let siblings = core.ordered_values(entity, attribute);
                let idx = siblings.iter().position(|d| &d.id == id);
                if idx.is_none() {
                    // should not happen
                    warn!("unable to find current datom");
                    return;
                }

                let idx = idx.unwrap();
                let idx_other = match direction {
                    Direction::Up => {
                        if idx > 0 {
                            idx - 1
                        } else {
                            // cannot move up the first element
                            return;
                        }
                    }
                    Direction::Down => {
                        if idx + 1 < siblings.len() {
                            idx + 1
                        } else {
                            // cannot move down the last element
                            return;
                        }
                    }
                };
                let id_other = &siblings[idx_other].id;

                id_other.clone()
            };

            match direction {
                Direction::Up => Self::move_above(store, id, &id_other),
                Direction::Down => Self::move_above(store, &id_other, id),
            };
        });
    }

    fn move_above(store: &mut Store, ida: &Field, idb: &Field) {
        trace!("moving {:?} above {:?}", ida, idb);

        if let Some(a_to_b) = store
            .eav2(ida, &ids::A_AFTER)
            .and_then(|fields| fields.iter().find(|d| &d.value == idb))
        {
            let a_to_b = a_to_b.clone();
            store.remove_datom(&a_to_b);
        }

        let from_a = store
            .aev2(&ids::A_AFTER, ida)
            .cloned()
            .unwrap_or_else(HashSet::new);
        let from_b = store
            .aev2(&ids::A_AFTER, idb)
            .cloned()
            .unwrap_or_else(HashSet::new);
        let into_a = store
            .ave2(&ids::A_AFTER, ida)
            .cloned()
            .unwrap_or_else(HashSet::new);
        let into_b = store
            .ave2(&ids::A_AFTER, idb)
            .cloned()
            .unwrap_or_else(HashSet::new);

        // swap all arrows
        for mut d in from_a {
            // these are now coming from b
            store.remove_datom(&d);
            d.entity = idb.clone();
            store.add_datom(&d);
        }
        for mut d in from_b {
            // these are now coming from a
            store.remove_datom(&d);
            d.entity = ida.clone();
            store.add_datom(&d);
        }
        for mut d in into_a {
            // these are now coming into b
            store.remove_datom(&d);
            d.value = idb.clone();
            store.add_datom(&d);
        }
        for mut d in into_b {
            // these are now coming into a
            store.remove_datom(&d);
            d.value = ida.clone();
            store.add_datom(&d);
        }

        // now add the final arrow: b after a
        store.add_datom(&Datom::eav(idb.clone(), ids::A_AFTER.clone(), ida.clone()))
    }
}

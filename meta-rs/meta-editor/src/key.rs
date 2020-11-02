use std::{fmt::Debug, fs::File, io::BufWriter};

use druid_shell::{HotKey, KeyCode, KeyEvent, RawMods, SysMods};

use crate::core_layout::{core_layout_datoms, core_layout_entities, core_layout_languages};
use crate::editor::Editor;
use crate::f_layout::f_layout_entries;

// TODO: we use KeyHandler as Box<dyn KeyHandler> in the rest of the editor. The Rust does not allow
// multiple traits (e.g., Box<dyn KeyHandler + Debug>), so we stick Debug in here.
pub trait KeyHandler: Debug {
    /// Return `true` if key was successfully handled.
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool;
}

#[derive(Debug)]
pub struct GlobalKeys;

impl KeyHandler for GlobalKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if let Some(text) = key.text() {
            if !key.mods.alt
                && !key.mods.ctrl
                && !key.mods.meta
                && text.chars().all(|c| !c.is_control())
            {
                if editor.self_insert(text) {
                    return true;
                }
                if editor.complete(text) {
                    return true;
                }
                return true;
            }
        }

        if HotKey::new(None, KeyCode::ArrowLeft).matches(key)
            || HotKey::new(SysMods::Cmd, KeyCode::KeyH).matches(key)
        {
            editor.move_cursor(0, -1);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowUp).matches(key)
            || HotKey::new(SysMods::Cmd, KeyCode::KeyJ).matches(key)
        {
            editor.move_cursor(-1, 0);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowDown).matches(key)
            || HotKey::new(SysMods::Cmd, KeyCode::KeyK).matches(key)
        {
            editor.move_cursor(1, 0);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowRight).matches(key)
            || HotKey::new(SysMods::Cmd, KeyCode::KeyL).matches(key)
        {
            editor.move_cursor(0, 1);
            return true;
        }

        if HotKey::new(None, KeyCode::Tab).matches(key) {
            editor.goto_next_editable_cell();
            return true;
        }
        if HotKey::new(SysMods::Shift, KeyCode::Tab).matches(key) {
            editor.goto_prev_editable_cell();
            return true;
        }

        if HotKey::new(None, KeyCode::Backspace).matches(key) {
            editor.backspace();
            return true;
        }
        if HotKey::new(None, KeyCode::Delete).matches(key) {
            editor.delete();
            return true;
        }
        if HotKey::new(SysMods::Cmd, KeyCode::KeyN).matches(key) {
            editor.complete("");
            return true;
        }

        if HotKey::new(SysMods::Cmd, KeyCode::KeyS).matches(key) {
            let store = editor.store();
            let f = File::create("store.meta").unwrap();
            let writer = BufWriter::new(f);
            serde_json::to_writer_pretty(writer, store).unwrap();
            return true;
        }

        if HotKey::new(RawMods::Alt, KeyCode::Key1).matches(key) {
            editor.set_layout_fn(core_layout_datoms);
            return true;
        }
        if HotKey::new(RawMods::Alt, KeyCode::Key2).matches(key) {
            editor.set_layout_fn(core_layout_entities);
            return true;
        }
        if HotKey::new(RawMods::Alt, KeyCode::Key3).matches(key) {
            editor.set_layout_fn(core_layout_languages);
            return true;
        }
        if HotKey::new(RawMods::Alt, KeyCode::Key4).matches(key) {
            editor.set_layout_fn(f_layout_entries);
            return true;
        }

        false
    }
}

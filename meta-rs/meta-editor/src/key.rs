use druid_shell::{HotKey, KeyCode, KeyEvent, RawMods};

use crate::editor::Editor;

pub trait KeyHandler {
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
            || HotKey::new(RawMods::Ctrl, KeyCode::KeyH).matches(key)
        {
            editor.move_cursor(0, -1);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowUp).matches(key)
            || HotKey::new(RawMods::Ctrl, KeyCode::KeyJ).matches(key)
        {
            editor.move_cursor(-1, 0);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowDown).matches(key)
            || HotKey::new(RawMods::Ctrl, KeyCode::KeyK).matches(key)
        {
            editor.move_cursor(1, 0);
            return true;
        }
        if HotKey::new(None, KeyCode::ArrowRight).matches(key)
            || HotKey::new(RawMods::Ctrl, KeyCode::KeyL).matches(key)
        {
            editor.move_cursor(0, 1);
            return true;
        }

        if HotKey::new(None, KeyCode::Escape).matches(key) {
            editor.escape();
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
        if HotKey::new(RawMods::Ctrl, KeyCode::KeyN).matches(key)
            || HotKey::new(None, KeyCode::Tab).matches(key)
        {
            editor.complete("");
            return true;
        }

        false
    }
}

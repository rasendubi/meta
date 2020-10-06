use druid_shell::kurbo::{Insets, Rect, Size};
use druid_shell::{piet::Color, HotKey, KeyCode, KeyEvent};
use log::debug;
use meta_gui::{
    Constraint, Direction, Event, EventType, GuiContext, Inset, Layout, List, Scrollable, Scrolled,
    SubscriptionId,
};
use std::{cmp::Ordering, collections::HashMap};
use unicode_segmentation::UnicodeSegmentation;

use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_datoms;
use crate::layout::{cmp_priority, EditorCellPayload};
use meta_core::MetaCore;
use meta_pretty::{Path, RichDoc, SimpleDoc, SimpleDocKind};
use meta_store::{Datom, MetaStore};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CursorPosition {
    Inside {
        cell: SimpleDoc<EditorCellPayload>,
        offset: usize,
    },
    Between(SimpleDoc<EditorCellPayload>, SimpleDoc<EditorCellPayload>),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellPosition {
    row: usize,
    col: usize,
}

pub struct Editor {
    id: SubscriptionId,
    store: MetaStore,
    doc: RichDoc<EditorCellPayload>,
    paths: HashMap<RichDoc<EditorCellPayload>, Path>,
    layout: Vec<Vec<SimpleDoc<EditorCellPayload>>>,
    positions: HashMap<SimpleDoc<EditorCellPayload>, CellPosition>,
    cursor: Option<CursorPosition>,
    scroll: Scrollable,
}

impl Editor {
    pub fn new(id: SubscriptionId, store: MetaStore) -> Self {
        let core = MetaCore::new(&store);
        let rich_doc = core_layout_datoms(&core);
        let paths = rich_doc.pathify();
        let sdoc = meta_pretty::layout(&rich_doc, 80);

        let layout = layout_to_2d(&sdoc);
        let positions = enumerate(&layout);
        let pos = CellPosition { row: 0, col: 0 };
        let cursor = Editor::cell_position_to_cursor(&positions, &layout, &pos);

        Editor {
            id,
            store,
            doc: rich_doc,
            paths,
            layout,
            positions,
            cursor,
            scroll: Scrollable::new(SubscriptionId::new()),
        }
    }

    pub fn current_position(&self) -> Option<CellPosition> {
        match &self.cursor {
            Some(CursorPosition::Inside { cell, offset }) => {
                let cell_position = self.positions.get(cell).unwrap();
                Some(CellPosition {
                    row: cell_position.row,
                    col: cell_position.col + offset,
                })
            }
            _ => None,
        }
    }

    pub fn on_store_updated(&mut self) {
        let core = MetaCore::new(&self.store);
        let rich_doc = core_layout_datoms(&core);
        let paths = rich_doc.pathify();
        let sdoc = meta_pretty::layout(&rich_doc, 80);

        let layout = layout_to_2d(&sdoc);
        let positions = enumerate(&layout);

        let cursor = if let Some(CursorPosition::Inside { cell, offset }) = &self.cursor {
            match rich_doc.follow_path(self.paths.get(cell.rich_doc()).unwrap()) {
                Ok(cell) => {
                    sdoc.iter()
                        .find(|s| s.rich_doc() == cell)
                        .map(|cell| CursorPosition::Inside {
                            cell: cell.clone(),
                            offset: *offset,
                        })
                }
                Err((_cell, _path)) => {
                    // TODO: The target cell has been deleted. Make cursor point to adjusted cell.
                    Editor::cell_position_to_cursor(
                        &positions,
                        &layout,
                        &self.current_position().unwrap(),
                    )
                }
            }
        } else {
            Editor::cell_position_to_cursor(&positions, &layout, &self.current_position().unwrap())
        };

        self.paths = paths;
        self.doc = rich_doc;
        self.layout = layout;
        self.cursor = cursor;
        self.positions = positions;
    }

    fn move_cursor(&mut self, drow: isize, dcol: isize) {
        let pos = self.current_position().unwrap();
        let pos = CellPosition {
            row: (pos.row as isize + drow) as usize,
            col: (pos.col as isize + dcol) as usize,
        };
        let cursor = Self::cell_position_to_cursor(&self.positions, &self.layout, &pos);

        self.cursor = cursor;
    }

    fn cell_position_to_cursor(
        positions: &HashMap<SimpleDoc<EditorCellPayload>, CellPosition>,
        layout: &[Vec<SimpleDoc<EditorCellPayload>>],
        pos: &CellPosition,
    ) -> Option<CursorPosition> {
        let CellPosition { row, col } = pos;
        layout
            .get(*row)
            .and_then(|r| {
                let m = r.iter().try_fold(None, |acc: Option<&SimpleDoc<_>>, cell| {
                    let left = positions.get(cell).unwrap().col;
                    let right = left + cell.width();
                    if *col < left || right <= *col {
                        Ok(Some(cell))
                    } else if left == *col {
                        Err(match acc {
                            None => CursorPosition::Inside {
                                cell: cell.clone(),
                                offset: col - left,
                            },
                            Some(prev) => CursorPosition::Between(prev.clone(), cell.clone()),
                        })
                    } else {
                        // strictly inside cell
                        Err(CursorPosition::Inside {
                            cell: cell.clone(),
                            offset: col - left,
                        })
                    }
                });
                match m {
                    Err(position) => Some(position),
                    Ok(mcell) => mcell.map(|cell| CursorPosition::Inside {
                        cell: cell.clone(),
                        offset: cell.width(),
                    }),
                }
            })
            .map(Self::resolve_cursor_priority)
    }

    fn resolve_cursor_priority(cursor: CursorPosition) -> CursorPosition {
        match cursor {
            CursorPosition::Inside { .. } => cursor,
            CursorPosition::Between(left, right) => match cmp_priority(&left, &right) {
                Ordering::Less | Ordering::Equal => CursorPosition::Inside {
                    cell: right,
                    offset: 0,
                },
                Ordering::Greater => CursorPosition::Inside {
                    offset: left.width(),
                    cell: left,
                },
            },
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.key_code {
            KeyCode::ArrowLeft => self.move_cursor(0, -1),
            KeyCode::ArrowUp => self.move_cursor(-1, 0),
            KeyCode::ArrowDown => self.move_cursor(1, 0),
            KeyCode::ArrowRight => self.move_cursor(0, 1),
            _ => {
                if let Some(text) = key.text() {
                    if !key.mods.alt
                        && !key.mods.ctrl
                        && !key.mods.meta
                        && text.chars().all(|c| !c.is_control())
                    {
                        self.self_insert(text);
                        return;
                    }
                }
                if HotKey::new(None, KeyCode::Backspace).matches(key) {
                    self.backspace();
                    return;
                }
                if HotKey::new(None, KeyCode::Delete).matches(key) {
                    self.delete();
                    return;
                }
            }
        }
    }

    fn self_insert(&mut self, text: &str) {
        let edited = self.edit_datom(|datom, offset| {
            let grapheme_offset = datom
                .value
                .as_ref()
                .grapheme_indices(true)
                .nth(offset)
                .map_or(datom.value.as_ref().len(), |x| x.0);

            let mut new_value = datom.value.to_string();
            new_value.insert_str(grapheme_offset, text);

            let mut new_datom = datom.clone();
            new_datom.value = new_value.into();

            Some(new_datom)
        });
        if edited {
            self.move_cursor(0, 1);
        }
    }

    fn backspace(&mut self) {
        let edited = self.edit_datom(|datom, offset| {
            let mut new_value = datom.value.to_string();
            if offset == 0 {
                return None;
            }

            let grapheme_offset = datom
                .value
                .as_ref()
                .grapheme_indices(true)
                .nth(offset - 1)
                .map(|x| x.0)
                .unwrap();

            new_value.remove(grapheme_offset);

            let mut new_datom = datom.clone();
            new_datom.value = new_value.into();

            Some(new_datom)
        });
        if edited {
            self.move_cursor(0, -1);
        }
    }

    fn delete(&mut self) {
        self.edit_datom(|datom, offset| {
            let grapheme_offset = datom
                .value
                .as_ref()
                .grapheme_indices(true)
                .nth(offset)
                .map_or(datom.value.as_ref().len(), |x| x.0);
            if grapheme_offset >= datom.value.as_ref().len() {
                return None;
            }

            let mut new_value = datom.value.to_string();
            new_value.remove(grapheme_offset);

            let mut new_datom = datom.clone();
            new_datom.value = new_value.into();

            Some(new_datom)
        });
    }

    /// Returns `true` if edit happened
    fn edit_datom<F: FnOnce(&Datom, usize) -> Option<Datom>>(&mut self, f: F) -> bool {
        if let Some(CursorPosition::Inside { cell, offset }) = &self.cursor {
            if let SimpleDocKind::Cell(cell) = cell.kind() {
                if let Some(datom) = &cell.payload.datom {
                    if let Some(new_datom) = f(datom, *offset) {
                        debug!("replacing {:?} with {:?}", datom, new_datom);

                        self.store.remove_datom(datom);
                        self.store.add_datom(&new_datom);
                        self.on_store_updated();

                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Layout for Editor {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        ctx.clear(Color::WHITE);

        let cursor = &self.cursor;
        let scroll = &mut self.scroll;
        let layout = &self.layout;

        Scrolled::new(
            scroll,
            &mut Inset::new(
                &mut List::new(layout.iter().map(|line| {
                    List::new(line.iter().map(|x| CellWidget(x, &cursor)))
                        .with_direction(Direction::Horizontal)
                })),
                Insets::uniform(10.0),
            ),
        )
        .layout(ctx, Constraint::tight(ctx.window_size()));

        ctx.grab_focus(self.id);
        ctx.subscribe(
            self.id,
            Rect::ZERO,
            EventType::FOCUS | EventType::KEY_DOWN,
            false,
        );

        for x in ctx.events(self.id) {
            debug!("Editor got event: {:?}", x);
            #[allow(clippy::single_match)]
            match x {
                Event::KeyDown(key) => self.handle_key(key),
                _ => {}
            }
            ctx.invalidate();
        }

        constraint.max
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WithEnumerate<M> {
    pos: CellPosition,
    meta: M,
}

fn enumerate<T>(layout: &[Vec<SimpleDoc<T>>]) -> HashMap<SimpleDoc<T>, CellPosition> {
    let mut result = HashMap::new();

    for (row_id, row) in layout.iter().enumerate() {
        let mut column = 0;
        for cell in row.iter() {
            result.insert(
                cell.clone(),
                CellPosition {
                    row: row_id,
                    col: column,
                },
            );
            column += cell.width();
        }
    }

    result
}

fn layout_to_2d<T>(layout: &[SimpleDoc<T>]) -> Vec<Vec<SimpleDoc<T>>> {
    let mut result = vec![Vec::new()];

    for cell in layout.iter() {
        if let SimpleDocKind::Linebreak { .. } = cell.kind() {
            result.push(Vec::new());
        }

        let last = result.len() - 1;
        unsafe { result.get_unchecked_mut(last) }.push(cell.clone());
    }

    result
}

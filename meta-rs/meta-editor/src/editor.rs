use std::cmp::Ordering;
use std::collections::HashMap;

use druid_shell::kurbo::{Insets, Rect, Size, Vec2};
use druid_shell::piet::Color;
use druid_shell::KeyEvent;
use im::HashSet;
use log::{debug, trace};
use unicode_segmentation::UnicodeSegmentation;

use meta_core::MetaCore;
use meta_gui::widgets::{Direction, Inset, List, Scrollable, Scrolled, Stack, Translate};
use meta_gui::{Constraint, Event, EventType, GuiContext, Layout, SubscriptionId};
use meta_pretty::{Path, SimpleDoc, SimpleDocKind};
use meta_store::{Datom, Field, Store};

use crate::autocomplete::{Autocomplete, AutocompleteEvent};
use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_languages;
use crate::key::{GlobalKeys, KeyHandler};
use crate::layout::{cmp_priority, CellClass, Doc, EditorCellPayload, RDoc, SDoc};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CursorPosition {
    Inside { cell: SDoc, offset: usize },
    // TODO: drop Between as it is virtually never used
    Between(SDoc, SDoc),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellPosition {
    row: usize,
    col: usize,
}

pub struct Editor {
    id: SubscriptionId,
    store: Store,
    doc: Doc,
    paths: HashMap<Doc, Path>,
    layout: Vec<Vec<SDoc>>,
    positions: HashMap<SDoc, CellPosition>,
    cursor: Option<CursorPosition>,
    scroll: Scrollable,
    autocomplete: Option<Translate<Autocomplete<Field>>>,
    layout_fn: fn(&Store) -> RDoc,
}

impl Editor {
    pub fn new(id: SubscriptionId, store: Store) -> Self {
        let layout_fn = core_layout_languages;
        let rich_doc: Doc = layout_fn(&store).into();
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
            autocomplete: None,
            layout_fn,
        }
    }

    pub fn set_layout_fn(&mut self, f: fn(&Store) -> RDoc) {
        self.layout_fn = f;
        self.on_store_updated();
        self.cursor = Editor::cell_position_to_cursor(
            &self.positions,
            &self.layout,
            &CellPosition { row: 0, col: 0 },
        );
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
        let rich_doc: Doc = (self.layout_fn)(&self.store).into();
        let paths = rich_doc.pathify();
        let sdoc = meta_pretty::layout(&rich_doc, 80);

        let layout = layout_to_2d(&sdoc);
        let positions = enumerate(&layout);

        let cursor = if let Some(CursorPosition::Inside { cell, offset }) = &self.cursor {
            let path = self.paths.get(cell.rich_doc()).unwrap();
            match rich_doc.follow_path(path).last().unwrap() {
                Ok(cell) => {
                    trace!("successfully resolved path {:?}", path);
                    sdoc.iter()
                        .find(|s| s.rich_doc() == cell)
                        .map(|cell| CursorPosition::Inside {
                            cell: cell.clone(),
                            offset: *offset,
                        })
                        .or_else(|| {
                            // TODO: think of a better strategy.
                            //
                            // This case means that the path is present in the new RichDoc but is
                            // absent in SimpleDoc. This likely means that RichDoc is no longer a
                            // cell (likely an Empty case)
                            Editor::cell_position_to_cursor(
                                &positions,
                                &layout,
                                &CellPosition { row: 0, col: 0 },
                            )
                        })
                }
                Err((_cell, _path)) => {
                    trace!("unable to follow path: {:?} left {:?}", path, _path);
                    // TODO: The target cell has been deleted. Make cursor point to adjusted cell.
                    Editor::cell_position_to_cursor(
                        &positions,
                        &layout,
                        &self.current_position().unwrap(),
                    )
                    .or_else(|| {
                        // TODO: think of a better strategy.
                        //
                        // This case likely means that we have deleted the very last item and cursor
                        // is now past the last row.
                        Editor::cell_position_to_cursor(
                            &positions,
                            &layout,
                            &CellPosition { row: 0, col: 0 },
                        )
                    })
                }
            }
        } else {
            trace!("no current cursor");
            Editor::cell_position_to_cursor(&positions, &layout, &self.current_position().unwrap())
        };

        self.paths = paths;
        self.doc = rich_doc;
        self.layout = layout;
        self.cursor = cursor;
        self.positions = positions;
    }

    pub fn move_cursor(&mut self, drow: isize, dcol: isize) {
        let pos = self.current_position().unwrap();
        let pos = CellPosition {
            row: (pos.row as isize + drow) as usize,
            col: (pos.col as isize + dcol) as usize,
        };
        let cursor = Self::cell_position_to_cursor(&self.positions, &self.layout, &pos);

        if cursor.is_some() {
            self.cursor = cursor;
        }
    }

    fn cell_position_to_cursor(
        positions: &HashMap<SDoc, CellPosition>,
        layout: &[Vec<SDoc>],
        pos: &CellPosition,
    ) -> Option<CursorPosition> {
        let CellPosition { row, col } = pos;
        layout
            .get(*row)
            .and_then(|r| {
                let m = r
                    .iter()
                    .try_fold(None, |acc: Option<&SimpleDoc<_, _>>, cell| {
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

    // needless_collect is false-positive here because clippy doesn't understand that we collect
    // to get a double-ended iterator.
    //
    // See https://github.com/rust-lang/rust-clippy/issues/5991#issuecomment-688224759
    #[allow(clippy::needless_collect)]
    fn handle_key(&mut self, key: KeyEvent) {
        let path = match &self.cursor {
            Some(CursorPosition::Inside {
                cell,
                offset: _offset,
            }) => self.paths.get(cell.rich_doc()).unwrap(),
            Some(CursorPosition::Between(..)) => panic!("cursor should not be Between"),
            None => panic!("cursor should not be None"),
        };

        let doc = self.doc.clone();
        let handlers = doc
            .follow_path(path)
            .filter_map(|x| x.ok())
            .filter_map(|x| x.as_meta())
            .collect::<Vec<_>>();
        for h in handlers.into_iter().rev() {
            if h.handle_key(key, self) {
                return;
            }
        }
        GlobalKeys.handle_key(key, self);
    }

    pub fn self_insert(&mut self, text: &str) -> bool {
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

        edited
    }

    pub fn backspace(&mut self) {
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

    pub fn delete(&mut self) {
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
                if let CellClass::Editable(datom) = &cell.payload.class {
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

    /// Returns `true` if current cell was a reference.
    pub fn complete(&mut self, input: &str) -> bool {
        if let Some(CursorPosition::Inside {
            cell: sdoc,
            offset: _,
        }) = &self.cursor
        {
            if let SimpleDocKind::Cell(cell) = sdoc.kind() {
                if let CellClass::Reference(datom, target, type_filter) = &cell.payload.class {
                    let candidates = self.candidates(input);

                    let position = self
                        .positions
                        .get(sdoc)
                        .copied()
                        .expect("complete: get cell position");

                    trace!(
                        "datom: {:?}, target: {:?}, type_filter: {:?}, candidates: {:?}",
                        datom,
                        target,
                        type_filter,
                        candidates
                    );

                    let CellPosition { row, col } = position;
                    let offset =
                        Self::cell_position_to_screen_offset(CellPosition { row: row + 1, col });
                    self.autocomplete = Some(Translate::new(
                        Autocomplete::new(SubscriptionId::new(), candidates)
                            .with_input(input.to_string()),
                        offset,
                    ));

                    return true;
                }
            }
        }

        false
    }

    fn candidates(&self, input: &str) -> Vec<(Field, String)> {
        if let Some(CursorPosition::Inside {
            cell: sdoc,
            offset: _,
        }) = &self.cursor
        {
            if let SimpleDocKind::Cell(cell) = sdoc.kind() {
                if let CellClass::Reference(_datom, _target, type_filter) = &cell.payload.class {
                    let core = MetaCore::new(&self.store);
                    let candidates: HashSet<Field> = match type_filter.filter() {
                        None => core.store.entities().into_iter().cloned().collect(),
                        Some(filter) => HashSet::unions(filter.iter().map(|type_| {
                            core.of_type(&type_)
                                .into_iter()
                                .map(|datom| datom.entity)
                                .collect()
                        })),
                    };

                    let input = input.to_lowercase();

                    let mut candidates = candidates
                        .iter()
                        .map(|id| {
                            (
                                id.clone(),
                                core.identifier(&id)
                                    .map_or(id, |datom| &datom.value)
                                    .to_string(),
                            )
                        })
                        .filter(|(i, s)| {
                            i.as_ref().contains(&input) || s.to_lowercase().contains(&input)
                        })
                        .collect::<Vec<_>>();

                    candidates.sort_unstable();
                    return candidates;
                }
            }
        }

        Vec::new()
    }

    fn finish_completion(&mut self, selection: Field) {
        if let Some(CursorPosition::Inside {
            cell: sdoc,
            offset: _,
        }) = &self.cursor
        {
            if let SimpleDocKind::Cell(cell) = sdoc.kind() {
                if let CellClass::Reference(datom, target, _type_filter) = &cell.payload.class {
                    let old_datom = datom;
                    let mut new_datom = datom.clone();
                    *target.get_field_mut(&mut new_datom) = selection;

                    trace!("replacing {:?} with {:?}", old_datom, new_datom);

                    self.store.remove_datom(old_datom);
                    self.store.add_datom(&new_datom);
                    self.on_store_updated();
                }
            }
        }
    }

    pub fn escape(&mut self) {
        if self.close_complete() {
            return;
        }
    }

    /// Returns `true` if completion was open.
    fn close_complete(&mut self) -> bool {
        self.autocomplete.take().is_some()
    }

    fn cell_position_to_screen_offset(pos: CellPosition) -> Vec2 {
        let CellPosition { row, col } = pos;
        let char_width = 6.0;
        let char_height = 12.0;
        let x_offset = col as f64 * char_width;
        let y_offset = row as f64 * char_height;
        Vec2::new(x_offset, y_offset)
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn with_store<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Store) -> R,
    {
        let result = f(&mut self.store);
        self.on_store_updated();
        result
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
            Inset::new(
                Stack::new(
                    [
                        &mut List::new(layout.iter().map(|line| {
                            List::new(line.iter().map(|x| CellWidget(x, &cursor)))
                                .with_direction(Direction::Horizontal)
                        })) as &mut dyn Layout,
                        &mut self.autocomplete,
                    ]
                    .iter_mut(),
                ),
                Insets::uniform(10.0),
            ),
        )
        .layout(ctx, Constraint::tight(ctx.window_size()));

        if let Some(autocomplete) = &mut self.autocomplete {
            for e in autocomplete.child_mut().events() {
                match e {
                    AutocompleteEvent::Close(e) => {
                        debug!("Autocomplete close with: {:?}", e);

                        if let Some((selection, _)) = e {
                            self.finish_completion(selection);
                        }

                        self.close_complete();
                        ctx.invalidate();
                    }

                    AutocompleteEvent::InputChanged(input) => {
                        let candidates = self.candidates(&input);
                        self.autocomplete
                            .as_mut()
                            .unwrap()
                            .child_mut()
                            .set_candidates(candidates);
                        ctx.invalidate();
                    }
                }
            }
        }

        ctx.grab_focus(self.id);
        ctx.subscribe(
            self.id,
            Rect::ZERO,
            EventType::FOCUS | EventType::KEY_DOWN,
            false,
        );

        for x in ctx.events(self.id) {
            trace!("Editor got event: {:?}", x);
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

fn enumerate<T, M>(layout: &[Vec<SimpleDoc<T, M>>]) -> HashMap<SimpleDoc<T, M>, CellPosition> {
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

fn layout_to_2d<T, M>(layout: &[SimpleDoc<T, M>]) -> Vec<Vec<SimpleDoc<T, M>>> {
    let mut result = vec![Vec::new()];

    for cell in layout.iter() {
        if let SimpleDocKind::Linebreak { .. } = cell.kind() {
            result.push(Vec::new());
        }

        result
            .last_mut()
            .expect("layout_to_2d: last_mut")
            .push(cell.clone());
    }

    result
}

#[allow(dead_code)]
fn simple_doc_to_string(sdoc: &[SimpleDoc<EditorCellPayload>]) -> String {
    let mut out = String::new();

    for doc in sdoc {
        match doc.kind() {
            SimpleDocKind::Linebreak { indent_width } => {
                out.reserve(indent_width + 1);
                out.push('\n');
                for _ in 0..*indent_width {
                    out.push(' ');
                }
            }
            SimpleDocKind::Cell(cell) => {
                out.push_str(cell.payload.text.as_ref());
            }
        }
    }

    out
}

use std::fmt::Debug;

use druid_shell::kurbo::{Insets, Point, Rect, Size, Vec2};
use druid_shell::piet::Color;
use druid_shell::{KeyEvent, MouseEvent};
use im::HashSet;
use itertools::Itertools;
use log::{debug, log_enabled, trace, warn, Level};
use unicode_segmentation::UnicodeSegmentation;

use meta_core::MetaCore;
use meta_gui::widgets::{Direction, List, Scrollable, ScrolledList, Translate};
use meta_gui::{Constraint, Event, EventType, GuiContext, Layout, SubscriptionId};
use meta_pretty::{Cell, Path, RichDocRef, SimpleDocKind};
use meta_store::{Datom, Field, Store};

use crate::autocomplete::{Autocomplete, AutocompleteEvent};
use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_languages;
use crate::doc_view::DocView;
use crate::key::{GlobalKeys, KeyHandler};
use crate::layout::{CellClass, Doc, EditorCellPayload, RDoc, SDoc};

const CHAR_HEIGHT: f64 = 12.0;
const CHAR_WIDTH: f64 = 6.0;

const INSET: f64 = 10.0;

const SCROLLOFF: f64 = 28.0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CursorPosition {
    pub sdoc: SDoc,
    pub offset: usize,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellPosition {
    pub row: usize,
    pub col: usize,
}
impl CellPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

pub struct Editor {
    id: SubscriptionId,
    store: Store,
    doc_view: DocView,
    cursor: Option<CursorPosition>,
    try_adjust_scroll: bool,
    scroll: Scrollable,
    autocomplete: Option<Translate<Autocomplete<Field>>>,
    layout_fn: fn(&Store) -> RDoc,
}

impl Editor {
    pub fn new(id: SubscriptionId, store: Store) -> Self {
        let layout_fn = core_layout_languages;
        let doc_view = DocView::new(layout_fn(&store).into());
        let cursor = doc_view.cell_position_to_cursor(CellPosition::new(0, 0));

        Editor {
            id,
            store,
            doc_view,
            cursor,
            try_adjust_scroll: true,
            scroll: Scrollable::new(SubscriptionId::new()),
            autocomplete: None,
            layout_fn,
        }
    }

    pub fn doc(&self) -> &Doc {
        &self.doc_view.doc()
    }

    pub fn rdoc_to_sdoc(&self, rdoc: &Doc) -> Option<&SDoc> {
        self.doc_view.rdoc_to_sdoc(rdoc)
    }

    fn get_node_path(&self, rdoc: &Doc) -> Option<&Path> {
        self.doc_view.get_node_path(rdoc)
    }

    fn cell_position_to_cursor(&self, pos: CellPosition) -> Option<CursorPosition> {
        self.doc_view.cell_position_to_cursor(pos)
    }

    pub fn set_cursor(&mut self, cursor: Option<CursorPosition>) {
        if log_enabled!(target: "cursor", Level::Trace) {
            trace!(
                target: "cursor",
                "set_cursor: {:#?}, path: {:?}",
                cursor,
                cursor
                    .as_ref()
                    .and_then(|cursor| self.get_node_path(cursor.sdoc.rich_doc()))
            );
        }
        self.cursor = cursor;
        self.try_adjust_scroll = true;
    }

    pub fn set_layout_fn(&mut self, f: fn(&Store) -> RDoc) {
        self.layout_fn = f;
        self.on_store_updated();
        self.set_cursor(self.cell_position_to_cursor(CellPosition::new(0, 0)));
    }

    pub fn current_position(&self) -> Option<CellPosition> {
        self.cursor.as_ref().map(|CursorPosition { sdoc, offset }| {
            let position = self.doc_view.get_sdoc_position(sdoc).unwrap();
            CellPosition::new(position.row, position.col + offset)
        })
    }

    pub fn on_store_updated(&mut self) {
        let doc_view = DocView::new((self.layout_fn)(&self.store).into());

        let cursor = self.cursor.as_ref().and_then(|CursorPosition { sdoc: s, offset }| {
            let old_path = self.get_node_path(s.rich_doc()).unwrap();
            match doc_view.doc().follow_path(old_path).last().unwrap() {
                Ok(cell) => {
                    trace!(target: "cursor", "successfully resolved path {:?}", old_path);
                    doc_view
                        .rdoc_to_sdoc(cell)
                        .map(|s| CursorPosition {
                            sdoc: s.clone(),
                            offset: *offset,
                        })
                    // TODO: think of a better strategy.
                    //
                    // .or_else case means that the path is present in the new RichDoc but is
                    // absent in SimpleDoc. This likely means that RichDoc is no longer a
                    // cell (likely an Empty case)
                }
                Err((_cell, _path)) => {
                    trace!(target: "cursor", "unable to follow path: {:?} left {:?}", old_path, _path);
                    // TODO: The target cell has been deleted. Make cursor point to adjusted cell.
                    None
                }
            }
        }).or_else(|| {
            trace!(target: "cursor", "cursor cannot be resolved, trying to set cursor to the same position");
            self.current_position().and_then(|pos| doc_view.cell_position_to_cursor(pos))
        }).or_else(|| {
            trace!(target: "cursor", "nothing worked: resetting cursor to (0,0) as a last resort");
            // TODO: think of a better strategy.
            //
            // This case likely means that we have deleted the very last item and cursor
            // is now past the last row.
            doc_view.cell_position_to_cursor(CellPosition::new(0, 0))
        });

        self.doc_view = doc_view;
        self.set_cursor(cursor);
    }

    pub fn move_cursor(&mut self, drow: isize, dcol: isize) {
        let pos = self.current_position().unwrap();
        let pos = CellPosition::new(
            (pos.row as isize + drow) as usize,
            (pos.col as isize + dcol) as usize,
        );
        let cursor = self.cell_position_to_cursor(pos);

        if cursor.is_some() {
            self.set_cursor(cursor);
        }
    }

    pub fn goto_next_editable_cell(&mut self) {
        if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
            let (row_id, i) = self.doc_view.find_sdoc(sdoc).expect("can't find sdoc");
            let layout = self.doc_view.layout();

            let next_cell = layout[row_id..]
                .iter()
                .enumerate()
                .find_map(|(row_id, row)| {
                    let row = if row_id == 0 { &row[i + 1..] } else { &row[..] };
                    row.iter().find(|sdoc| {
                        sdoc.as_cell().map_or(false, |cell| {
                            matches!(cell.payload.class,
                                CellClass::Reference(_, _, _) | CellClass::Editable(_))
                        })
                    })
                });

            if let Some(next_cell) = next_cell.cloned() {
                self.set_cursor(Some(CursorPosition {
                    sdoc: next_cell,
                    offset: 0,
                }));
            }
        }
    }

    pub fn goto_prev_editable_cell(&mut self) {
        if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
            let (row_id, i) = self.doc_view.find_sdoc(sdoc).expect("can't find sdoc");
            let layout = self.doc_view.layout();

            let next_cell = layout[..=row_id]
                .iter()
                .rev()
                .enumerate()
                .find_map(|(row_id, row)| {
                    let row = if row_id == 0 { &row[..i] } else { &row[..] };
                    row.iter().rev().find(|sdoc| {
                        sdoc.as_cell().map_or(false, |cell| {
                            matches!(cell.payload.class,
                                CellClass::Reference(_, _, _) | CellClass::Editable(_))
                        })
                    })
                });

            if let Some(next_cell) = next_cell.cloned() {
                self.set_cursor(Some(CursorPosition {
                    sdoc: next_cell,
                    offset: 0,
                }));
            }
        }
    }

    // needless_collect is false-positive here because clippy doesn't understand that we collect
    // to get a double-ended iterator.
    //
    // See https://github.com/rust-lang/rust-clippy/issues/5991#issuecomment-688224759
    #[allow(clippy::needless_collect)]
    fn handle_key(&mut self, key: KeyEvent) {
        let path = match &self.cursor {
            Some(CursorPosition {
                sdoc,
                offset: _offset,
            }) => self.get_node_path(sdoc.rich_doc()).unwrap(),
            None => panic!("cursor should not be None"),
        };

        let doc = self.doc().clone();

        let global_keys = GlobalKeys;
        let mut handlers = vec![&global_keys as &dyn KeyHandler];
        handlers.extend(
            doc.follow_path(path)
                .filter_map(|x| x.ok())
                .filter_map(|x| x.as_meta().and_then(|m| m.key_handler())),
        );
        let handlers = handlers;

        trace!(target: "keys", "handlers: {:?}", handlers);
        for h in handlers.into_iter().rev() {
            if h.handle_key(key, self) {
                trace!(target: "keys", "found key in {:?}", h);
                self.try_adjust_scroll = true;
                return;
            }
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let inset: Vec2 = (INSET, INSET).into();
        let pos = Self::screen_offset_to_position(mouse.pos - inset + self.scroll.offset());
        let cursor = self.cell_position_to_cursor(pos);
        self.set_cursor(cursor);
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
        if let Some(CursorPosition { sdoc, offset }) = &self.cursor {
            if let SimpleDocKind::Cell(cell) = sdoc.kind() {
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
        if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
            if let SimpleDocKind::Cell(cell) = sdoc.kind() {
                if let CellClass::Reference(datom, target, type_filter) = &cell.payload.class {
                    let candidates = self.candidates(input);

                    let position = self
                        .doc_view
                        .get_sdoc_position(sdoc)
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
                        Self::cell_position_to_screen_offset(CellPosition::new(row + 1, col));
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
        if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
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

                    let candidates = candidates
                        .iter()
                        .map(|id| {
                            let type_ = core.meta_type(id).map(|d| &d.value);

                            let display = type_
                                .and_then(|type_| {
                                    // TODO: how to decouple type-specific display?
                                    if type_ == &meta_f::ids::IDENTIFIER as &Field {
                                        core.store
                                            .value(id, &meta_f::ids::IDENTIFIER_IDENTIFIER)
                                            .map(|d| &d.value)
                                    } else {
                                        None
                                    }
                                })
                                .or_else(|| core.identifier(&id).map(|datom| &datom.value))
                                .unwrap_or(id)
                                .to_string();

                            (id.clone(), display)
                        })
                        .filter_map(|(id, s)| s.to_lowercase().find(&input).map(|i| (i, s, id)))
                        .sorted()
                        .map(|(_i, s, id)| (id, s))
                        .collect::<Vec<_>>();

                    return candidates;
                }
            }
        }

        Vec::new()
    }

    fn finish_completion(&mut self, selection: Field) {
        if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
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

    /// Returns `true` if completion was open.
    fn close_complete(&mut self) -> bool {
        self.autocomplete.take().is_some()
    }

    fn cell_position_to_screen_offset(pos: CellPosition) -> Vec2 {
        let CellPosition { row, col } = pos;
        let x_offset = col as f64 * CHAR_WIDTH;
        let y_offset = row as f64 * CHAR_HEIGHT;
        Vec2::new(x_offset, y_offset)
    }

    fn screen_offset_to_position(point: Point) -> CellPosition {
        let Point { x, y } = point;
        CellPosition::new(
            (y / CHAR_HEIGHT) as usize,
            (x / CHAR_WIDTH).round() as usize,
        )
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

    pub fn goto_cell_id(&mut self, id: &[Field]) {
        if let Some(doc) = find_id(self.doc(), id).and_then(|doc| {
            find_cell(doc, &mut |cell: &Cell<EditorCellPayload>| {
                matches!(&cell.payload.class,
                    CellClass::Reference(_, _, _)
                    | CellClass::Editable(_)
                    | CellClass::NonEditable)
            })
        }) {
            if let Some(sdoc) = self.rdoc_to_sdoc(doc).cloned() {
                self.set_cursor(Some(CursorPosition { sdoc, offset: 0 }));
            }
        } else {
            warn!("cell with id {:?} not found", id);
        }
    }
}

impl Layout for Editor {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        ctx.clear(Color::WHITE);

        if self.try_adjust_scroll {
            trace!(target: "scroll", "adjusting scroll");
            if let Some(CursorPosition { .. }) = &self.cursor {
                if let Some(pos) = self.current_position() {
                    let offset: Vec2 =
                        Self::cell_position_to_screen_offset(pos) + Vec2::new(INSET, INSET);
                    let cell_rect = Rect::from_origin_size(
                        offset.to_point(),
                        Size::new(CHAR_WIDTH, CHAR_HEIGHT),
                    )
                    .inset(SCROLLOFF);

                    let scroll_offset = self.scroll.offset();
                    let screen_rect =
                        Rect::from_origin_size(scroll_offset.to_point(), ctx.window_size());

                    trace!(target: "scroll", "pos: {:?}, cell_rect: {:?}, screen_rect: {:?}", pos, cell_rect, screen_rect);

                    let mut target_rect = screen_rect;
                    if cell_rect.x1 > target_rect.x1 {
                        target_rect = target_rect + Vec2::new(cell_rect.x1 - target_rect.x1, 0.0);
                    }
                    if cell_rect.x0 < target_rect.x0 {
                        target_rect = target_rect - Vec2::new(target_rect.x0 - cell_rect.x0, 0.0);
                    }
                    if cell_rect.y1 > target_rect.y1 {
                        target_rect = target_rect + Vec2::new(0.0, cell_rect.y1 - target_rect.y1);
                    }
                    if cell_rect.y0 < target_rect.y0 {
                        target_rect = target_rect - Vec2::new(0.0, target_rect.y0 - cell_rect.y0);
                    }

                    let target_scroll = target_rect.origin().to_vec2();
                    trace!(target: "scroll", "current scroll: {:?}, target scroll: {:?}", scroll_offset, target_scroll);
                    self.scroll.set_offset(target_scroll);
                }
            }

            self.try_adjust_scroll = false;
        }

        let cursor = &self.cursor;
        let scroll = &mut self.scroll;
        let layout = self.doc_view.layout();

        ScrolledList::new(
            scroll,
            CHAR_HEIGHT,
            layout.iter().map(|line| {
                List::new(line.iter().map(|x| CellWidget(x, &cursor)))
                    .with_direction(Direction::Horizontal)
            }),
        )
        .with_insets(Insets::uniform(INSET))
        .layout(ctx, Constraint::tight(ctx.window_size()));

        Translate::new(
            &mut self.autocomplete,
            -scroll.offset() + Vec2::new(INSET, INSET),
        )
        .layout(ctx, Constraint::loose(ctx.window_size()));

        if let Some(autocomplete) = &mut self.autocomplete {
            for e in autocomplete.child_mut().events() {
                match e {
                    AutocompleteEvent::Close(e) => {
                        debug!("Autocomplete close with: {:?}", e);

                        if let Some((selection, _)) = e {
                            self.finish_completion(selection);
                        }

                        self.close_complete();
                        trace!("invalidate!");
                        ctx.invalidate();
                    }

                    AutocompleteEvent::InputChanged(input) => {
                        let candidates = self.candidates(&input);
                        self.autocomplete
                            .as_mut()
                            .unwrap()
                            .child_mut()
                            .set_candidates(candidates);
                        trace!("invalidate!");
                        ctx.invalidate();
                    }
                }
            }
        }

        ctx.grab_focus(self.id);
        ctx.subscribe(
            self.id,
            ctx.window_size().to_rect(),
            EventType::FOCUS | EventType::KEY_DOWN | EventType::MOUSE_DOWN,
            false,
        );

        for x in ctx.events(self.id) {
            trace!("Editor got event: {:?}", x);
            #[allow(clippy::single_match)]
            match x {
                Event::KeyDown(key) => self.handle_key(key),
                Event::MouseDown(mouse) => self.handle_mouse(mouse),
                _ => {}
            }
            trace!("invalidate!");
            ctx.invalidate();
        }

        constraint.max
    }
}

impl Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Editor")
            .field("id", &self.id)
            .field("store", &self.store)
            .field("cursor", &self.cursor)
            .field("scroll", &self.scroll)
            .field("autocomplete", &self.autocomplete)
            .finish()
    }
}

/// Find meta node with id `id`.
fn find_id<'a, 'b>(doc: &'a Doc, id: &'b [Field]) -> Option<&'a Doc> {
    match doc.kind() {
        meta_pretty::RichDocKind::Empty => None,
        meta_pretty::RichDocKind::Cell(_) => None,
        meta_pretty::RichDocKind::Line { alt: _ } => None,
        meta_pretty::RichDocKind::Nest { nest_width: _, doc } => find_id(doc, id),
        meta_pretty::RichDocKind::Concat { parts } => parts
            .iter()
            .fold(None, |acc, doc| acc.or_else(|| find_id(doc, id))),
        meta_pretty::RichDocKind::Group { doc } => find_id(doc, id),
        meta_pretty::RichDocKind::Meta {
            doc: nested_doc,
            meta,
        } => {
            if meta.id().map_or(false, |i| i.as_slice() == id) {
                Some(doc)
            } else {
                find_id(nested_doc, id)
            }
        }
    }
}

/// Find first cell matching the predicate in the doc.
fn find_cell<'a, T, M, F>(
    doc: &'a RichDocRef<T, M>,
    pred: &'_ mut F,
) -> Option<&'a RichDocRef<T, M>>
where
    F: FnMut(&Cell<T>) -> bool,
{
    match doc.kind() {
        meta_pretty::RichDocKind::Empty => None,
        meta_pretty::RichDocKind::Cell(cell) => {
            if pred(cell) {
                Some(doc)
            } else {
                None
            }
        }
        meta_pretty::RichDocKind::Line { alt: _ } => None,
        meta_pretty::RichDocKind::Nest { nest_width: _, doc } => find_cell(doc, pred),
        meta_pretty::RichDocKind::Concat { parts } => parts
            .iter()
            .fold(None, |acc, doc| acc.or_else(|| find_cell(doc, pred))),
        meta_pretty::RichDocKind::Group { doc } => find_cell(doc, pred),
        meta_pretty::RichDocKind::Meta { doc, meta: _ } => find_cell(doc, pred),
    }
}

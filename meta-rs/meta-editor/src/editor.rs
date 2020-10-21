use std::fmt::Debug;

use druid_shell::kurbo::{Insets, Rect, Size, Vec2};
use druid_shell::piet::Color;
use druid_shell::KeyEvent;
use im::HashSet;
use log::{debug, log_enabled, trace, Level};
use unicode_segmentation::UnicodeSegmentation;

use meta_core::MetaCore;
use meta_gui::widgets::{Direction, Inset, List, Scrollable, Scrolled, Stack, Translate};
use meta_gui::{Constraint, Event, EventType, GuiContext, Layout, SubscriptionId};
use meta_pretty::{Path, SimpleDocKind};
use meta_store::{Datom, Field, Store};

use crate::autocomplete::{Autocomplete, AutocompleteEvent};
use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_languages;
use crate::doc_view::DocView;
use crate::key::{GlobalKeys, KeyHandler};
use crate::layout::{CellClass, Doc, RDoc, SDoc};

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

        if self.try_adjust_scroll {
            trace!(target: "scroll", "adjusting scroll");
            if let Some(CursorPosition { sdoc, offset: _ }) = &self.cursor {
                if let Some(pos) = self.current_position() {
                    let scrolloff = 24.0;

                    let offset: Vec2 =
                        Self::cell_position_to_screen_offset(pos) + Vec2::new(10.0, 10.0);
                    let cell_rect = Rect::from_origin_size(
                        offset.to_point(),
                        Size::new(6.0 * sdoc.width() as f64, 12.0),
                    )
                    .inset(scrolloff);

                    let scroll_offset = self.scroll.offset();
                    let screen_rect =
                        Rect::from_origin_size(scroll_offset.to_point(), ctx.window_size());

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

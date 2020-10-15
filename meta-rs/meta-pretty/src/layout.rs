use crate::rich_doc::{RichDoc, RichDocKind};
use crate::simple_doc::SimpleDoc;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Mode {
    Break,
    Flat,
}

type Cmd<'a, T, M> = (usize, Mode, &'a RichDoc<T, M>);

fn fits<T, M>(cmd: Cmd<T, M>, rest: &[Cmd<T, M>], max_width: usize) -> bool {
    // Semantically, this function should take `cmds: Vec<Cmd<T>>`. However, that would imply a copy
    // of the cmds vector which we try to avoid.
    //
    // Create a separate cmds vector and pull commands from `rest` as needed.
    //
    // Another option would be to try im::Vector which offers O(1) copy.
    let mut rest_cmds = rest.iter().rev();
    let mut cmds = vec![cmd];

    let mut width = 0;
    while width <= max_width {
        match cmds.pop() {
            None => {
                // refill from `rest`
                if let Some(cmd) = rest_cmds.next() {
                    cmds.push(*cmd);
                } else {
                    return true;
                }
            }

            Some((indent, mode, doc)) => match doc.kind() {
                RichDocKind::Empty => {}
                RichDocKind::Cell(cell) => {
                    width += cell.width;
                }
                RichDocKind::Concat { parts } => {
                    for part in parts.iter().rev() {
                        cmds.push((indent, mode, part));
                    }
                }
                RichDocKind::Nest { nest_width, doc } => {
                    cmds.push((indent + nest_width, mode, doc));
                }
                RichDocKind::Line { alt } => {
                    if mode == Mode::Break {
                        return true;
                    }

                    if let Some(cell) = alt {
                        width += cell.width;
                    }
                }
                RichDocKind::Group { doc } => {
                    cmds.push((indent, Mode::Flat, doc));
                }
                RichDocKind::Meta { doc, meta: _meta } => cmds.push((indent, mode, doc)),
            },
        }
    }

    false
}

pub fn layout<T, M>(doc: &RichDoc<T, M>, page_width: usize) -> Vec<SimpleDoc<T, M>>
where
    // TODO: think of a way to lift off this `T: Clone` constraint. It is needed because cells are
    // currently copied to `SimpleDoc`, but that's not really necessary as `SimpleDoc` already holds
    // a reference to originating `RichDoc` (which owns `Cell`).
    T: Clone,
{
    let mut out = vec![];

    let mut cmds = vec![(0, Mode::Break, doc)];
    let mut pos = 0;

    while let Some((indent, mode, doc)) = cmds.pop() {
        match doc.kind() {
            RichDocKind::Empty => {}
            RichDocKind::Cell(cell) => {
                out.push(SimpleDoc::cell(doc.clone(), cell.clone()));
                pos += cell.width;
            }
            RichDocKind::Concat { parts } => {
                cmds.reserve(parts.len());
                for part in parts.iter().rev() {
                    cmds.push((indent, mode, part));
                }
            }
            RichDocKind::Line { alt } => match mode {
                Mode::Break => {
                    out.push(SimpleDoc::linebreak(doc.clone(), indent));
                    pos = indent;
                }
                Mode::Flat => {
                    if let Some(alt) = alt {
                        out.push(SimpleDoc::cell(doc.clone(), alt.clone()));
                    }
                }
            },
            RichDocKind::Nest { nest_width, doc } => {
                cmds.push((indent + nest_width, mode, &doc));
            }
            RichDocKind::Group { doc } => {
                let mode = if fits((indent, Mode::Flat, &doc), &cmds, page_width - pos) {
                    Mode::Flat
                } else {
                    Mode::Break
                };

                cmds.push((indent, mode, &doc));
            }
            RichDocKind::Meta { doc, meta: _meta } => {
                cmds.push((indent, mode, &doc));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::rich_doc::{Cell, RichDoc};
    use crate::simple_doc::{SimpleDoc, SimpleDocKind};

    fn text(s: &str) -> RichDoc<&str> {
        RichDoc::cell(Cell::new(s.len(), s))
    }

    fn to_string(sdoc: &Vec<SimpleDoc<&str>>) -> String {
        let mut result = String::new();

        for s in sdoc {
            match s.kind() {
                SimpleDocKind::Cell(Cell { payload, .. }) => {
                    result.push_str(payload);
                }
                SimpleDocKind::Linebreak { indent_width } => {
                    result.push('\n');
                    for _ in 0..*indent_width {
                        result.push(' ')
                    }
                }
            }
        }

        result
    }

    fn layout(doc: RichDoc<&str>) -> String {
        to_string(&super::layout(&doc, 20))
    }

    #[test]
    fn test_empty() {
        assert_eq!(layout(RichDoc::empty()), "");
    }

    #[test]
    fn test_cell() {
        assert_eq!(layout(text("hello")), "hello");
    }

    #[test]
    fn test_concat_cells() {
        assert_eq!(
            layout(RichDoc::concat(vec![
                text("hello,"),
                text(" "),
                text("world!")
            ])),
            "hello, world!"
        );
    }

    #[test]
    fn test_line() {
        assert_eq!(
            layout(RichDoc::concat(vec![
                text("hello"),
                RichDoc::linebreak(),
                text("world!")
            ])),
            "hello\nworld!"
        )
    }

    #[test]
    fn test_nest_and_line() {
        assert_eq!(
            layout(RichDoc::concat(vec![
                text("hello"),
                RichDoc::nest(
                    2,
                    RichDoc::concat(vec![RichDoc::linebreak(), text("world!")])
                )
            ])),
            "hello\n  world!"
        )
    }

    #[test]
    fn test_group_text() {
        assert_eq!(layout(RichDoc::group(text("blah"))), "blah");
    }

    #[test]
    fn test_group_line() {
        assert_eq!(layout(RichDoc::group(RichDoc::linebreak())), "");
    }

    #[test]
    fn test_group_line_alt() {
        assert_eq!(
            layout(RichDoc::group(RichDoc::line(Cell::new(1, ",")))),
            ","
        );
    }

    #[test]
    fn test_group_empty() {
        assert_eq!(layout(RichDoc::group(RichDoc::empty())), "");
    }

    #[test]
    fn test_group_flat() {
        assert_eq!(
            layout(RichDoc::group(RichDoc::concat(vec![
                text("text"),
                RichDoc::line(Cell::new(1, " ")),
                text("more text")
            ]))),
            "text more text"
        );
    }

    #[test]
    fn test_group_break() {
        assert_eq!(
            layout(RichDoc::group(RichDoc::concat(vec![
                text("long text"),
                RichDoc::line(Cell::new(1, " ")),
                text("more long text")
            ]))),
            "long text\nmore long text"
        );
    }
}

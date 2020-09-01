#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Cell<T> {
    pub width: usize,
    pub payload: T,
}

pub fn cell<T>(width: usize, payload: T) -> Cell<T> {
    Cell { width, payload }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RichDoc<T> {
    Empty,
    Cell(Cell<T>),
    Line {
        alt: Option<Cell<T>>,
        // TODO: hard-break
    },
    Nest {
        nest_width: usize,
        doc: Box<RichDoc<T>>,
    },
    Concat {
        parts: Vec<RichDoc<T>>
    },
    Group {
        doc: Box<RichDoc<T>>,
    }
}

impl<T> RichDoc<T> {
    #[inline]
    pub fn empty() -> Self {
        RichDoc::Empty
    }

    #[inline]
    pub fn cell(width: usize, payload: T) -> Self {
        RichDoc::Cell(Cell { width, payload })
    }

    #[inline]
    pub fn line(alt: Cell<T>) -> Self {
        RichDoc::Line { alt: Some(alt) }
    }

    #[inline]
    pub fn linebreak() -> Self {
        RichDoc::Line { alt: None }
    }

    #[inline]
    pub fn nest(width: usize, doc: RichDoc<T>) -> Self {
        RichDoc::Nest {
            nest_width: width,
            doc: Box::new(doc),
        }
    }

    #[inline]
    pub fn concat(parts: Vec<RichDoc<T>>) -> Self {
        RichDoc::Concat { parts }
    }

    #[inline]
    pub fn group(doc: RichDoc<T>) -> Self {
        RichDoc::Group {
            doc: Box::new(doc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let _empty = RichDoc::<&str>::empty();
    }

    #[test]
    fn test_cell() {
        let cell = RichDoc::cell(5, "hello");
        assert_eq!(RichDoc::Cell(Cell { width: 5, payload: "hello" }), cell);
    }

    #[test]
    fn test_complex() {
        let _doc =
            RichDoc::group(
                RichDoc::nest(
                    2,
                    RichDoc::concat(vec![
                        RichDoc::cell(2, 11),
                        RichDoc::empty(),
                        RichDoc::line(cell(2, 22)),
                        RichDoc::linebreak(),
                    ])));
    }
}

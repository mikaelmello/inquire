// sorry for this file

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub relative_index: usize,
    pub absolute_index: usize,
}

pub struct Page<'a, T> {
    pub first: bool,
    pub last: bool,
    pub content: &'a [T],
    pub cursor: Option<Cursor>,
    pub total: usize,
}

pub fn paginate<T>(page_size: usize, choices: &[T], sel: Option<usize>) -> Page<T> {
    // if there is no selection, we default to the first page.
    // in practice, the same as selecting the 0 index.

    let (start, end, cursor) = if choices.len() <= page_size {
        (0, choices.len(), sel.map(Cursor::same))
    } else if let Some(index) = sel {
        if index < page_size / 2 {
            // if we are in the first half page
            let start = 0;
            let end = page_size;
            let cursor = Some(Cursor::same(index));

            (start, end, cursor)
        } else if choices.len() - index - 1 < page_size / 2 {
            // if we are in the last half page
            let start = choices.len() - page_size;
            let end = choices.len();
            let cursor = Some(Cursor::new(index - start, index));

            (start, end, cursor)
        } else {
            // somewhere in the middle
            let above = page_size / 2;
            let below = page_size - above;

            let start = index - above;
            let end = index + below;
            let cursor = Some(Cursor::new(page_size / 2, index));

            (start, end, cursor)
        }
    } else {
        // if we are in the first half page
        let start = 0;
        let end = page_size;

        (start, end, sel.map(Cursor::same))
    };

    Page {
        first: start == 0,
        last: end == choices.len(),
        content: &choices[start..end],
        cursor,
        total: choices.len(),
    }
}

impl Cursor {
    fn new(relative_index: usize, absolute_index: usize) -> Cursor {
        Cursor {
            relative_index,
            absolute_index,
        }
    }

    fn same(idx: usize) -> Cursor {
        Cursor {
            relative_index: idx,
            absolute_index: idx,
        }
    }
}

pub fn int_log10<T>(mut i: T) -> usize
where
    T: std::ops::DivAssign + std::cmp::PartialOrd + From<u8> + Copy,
{
    let mut len = 0;
    let zero = T::from(0);
    let ten = T::from(10);

    while i > zero {
        i /= ten;
        len += 1;
    }

    len
}

#[cfg(test)]
mod test {
    #![allow(clippy::bool_assert_comparison)]

    use crate::{
        list_option::ListOption,
        utils::{int_log10, paginate, Cursor},
    };

    #[test]
    fn int_log10_works() {
        for i in 1..10 {
            assert_eq!(1, int_log10(i), "Int log 10 failed for value {i}");
        }
        for i in 10..100 {
            assert_eq!(2, int_log10(i), "Int log 10 failed for value {i}");
        }
        for i in 100..1000 {
            assert_eq!(3, int_log10(i), "Int log 10 failed for value {i}");
        }
        for i in 1000..10000 {
            assert_eq!(4, int_log10(i), "Int log 10 failed for value {i}");
        }
        for i in 10000..100000 {
            assert_eq!(5, int_log10(i), "Int log 10 failed for value {i}");
        }
    }

    #[test]
    fn paginate_too_few() {
        let choices = ListOption::from_list(vec!["1", "2", "3"]);

        let page_size = 4usize;
        let sel = Some(3usize);

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[..], page.content[..]);
        assert_eq!(Some(Cursor::new(3usize, 3)), page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(true, page.last);
        assert_eq!(3, page.total);
    }

    #[test]
    fn paginate_too_few_no_cursor() {
        let choices = ListOption::from_list(vec!["1", "2", "3"]);

        let page_size = 4usize;
        let sel = None;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[..], page.content[..]);
        assert_eq!(None, page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(true, page.last);
        assert_eq!(3, page.total);
    }

    #[test]
    fn paginate_first_half() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 4usize;
        let sel = Some(2usize);

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..4], page.content[..]);
        assert_eq!(Some(Cursor::new(2, 2)), page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_first_half_no_cursor() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 4usize;
        let sel = None;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..4], page.content[..]);
        assert_eq!(None, page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_middle() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 2usize;
        let sel = Some(3usize);

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[2..4], page.content[..]);
        assert_eq!(Some(Cursor::new(1, 3)), page.cursor);
        assert_eq!(false, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_middle_no_cursor() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 2usize;
        let sel = None;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..2], page.content[..]);
        assert_eq!(None, page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_last_half() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 3usize;
        let sel = Some(5usize);

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[3..6], page.content[..]);
        assert_eq!(Some(Cursor::new(2, 5)), page.cursor);
        assert_eq!(false, page.first);
        assert_eq!(true, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_last_half_no_cursor() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 3usize;
        let sel = None;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..3], page.content[..]);
        assert_eq!(None, page.cursor);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }
}

impl<'a, T> Debug for Page<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("first", &self.first)
            .field("last", &self.last)
            .field("content", &format!("({} elements)", &self.content.len()))
            .field("cursor", &self.cursor)
            .field("total", &self.total)
            .finish()
    }
}

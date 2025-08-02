// sorry for this file

use std::fmt::Debug;

pub struct Page<'a, T> {
    /// Whether this is the first page.
    pub first: bool,

    /// Whether this is the last page.
    pub last: bool,

    /// Content of the page.
    pub content: &'a [T],

    /// If a cursor exists on the original list, this is the index of the new cursor relative to the output list of choices, the page.
    pub cursor: Option<usize>,

    /// Total amount of elements in the original list of choices.
    pub total: usize,
}

pub fn paginate<T>(page_size: usize, choices: &[T], sel: Option<usize>) -> Page<'_, T> {
    // if there is no selection, we default to the first page.
    // in practice, the same as selecting the 0 index.

    let (start, end, cursor) = if choices.len() <= page_size {
        (0, choices.len(), sel)
    } else if let Some(index) = sel {
        if index < page_size / 2 {
            // if we are in the first half page
            let start = 0;
            let end = page_size;
            let cursor = Some(index);

            (start, end, cursor)
        } else if choices.len() - index - 1 < page_size / 2 {
            // if we are in the last half page
            let start = choices.len() - page_size;
            let end = choices.len();
            let cursor = Some(index - start);

            (start, end, cursor)
        } else {
            // somewhere in the middle
            let above = page_size / 2;
            let below = page_size - above;

            let start = index - above;
            let end = index + below;
            let cursor = Some(page_size / 2);

            (start, end, cursor)
        }
    } else {
        // if we are in the first half page
        let start = 0;
        let end = page_size;

        (start, end, sel)
    };

    Page {
        first: start == 0,
        last: end == choices.len(),
        content: &choices[start..end],
        cursor,
        total: choices.len(),
    }
}

pub fn int_log10<T>(mut i: T) -> usize
where
    T: std::ops::DivAssign + PartialOrd + From<u8> + Copy,
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

#[cfg(test)]
mod test {
    #![allow(clippy::bool_assert_comparison)]

    use crate::{
        list_option::ListOption,
        utils::{int_log10, paginate},
    };

    impl<T> ListOption<T> {
        pub(crate) fn from_list(vals: Vec<T>) -> Vec<ListOption<T>> {
            vals.into_iter()
                .enumerate()
                .map(|(index, value)| Self { index, value })
                .collect()
        }
    }

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
        assert_eq!(Some(3usize), page.cursor);
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
        assert_eq!(Some(2), page.cursor);
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
        assert_eq!(Some(1), page.cursor);
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
        assert_eq!(Some(2), page.cursor);
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

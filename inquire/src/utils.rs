// sorry for this file

pub struct Page<'a, T> {
    pub first: bool,
    pub last: bool,
    pub content: &'a [T],
    pub selection: usize,
    pub total: usize,
}

pub fn paginate<T>(page_size: usize, choices: &[T], sel: usize) -> Page<T> {
    let (start, end, cursor) = if choices.len() <= page_size {
        (0, choices.len(), sel)
    } else if sel < page_size / 2 {
        // if we are in the first half page
        let start = 0;
        let end = page_size;
        let cursor = sel;

        (start, end, cursor)
    } else if choices.len() - sel - 1 < page_size / 2 {
        // if we are in the last half page
        let start = choices.len() - page_size;
        let end = choices.len();
        let cursor = sel - start;

        (start, end, cursor)
    } else {
        // somewhere in the middle
        let above = page_size / 2;
        let below = page_size - above;

        let start = sel - above;
        let end = sel + below;
        let cursor = page_size / 2;

        (start, end, cursor)
    };

    Page {
        first: start == 0,
        last: end == choices.len(),
        content: &choices[start..end],
        selection: cursor,
        total: choices.len(),
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
        utils::{int_log10, paginate},
    };

    #[test]
    fn int_log10_works() {
        for i in 1..10 {
            assert_eq!(1, int_log10(i), "Int log 10 failed for value {}", i);
        }
        for i in 10..100 {
            assert_eq!(2, int_log10(i), "Int log 10 failed for value {}", i);
        }
        for i in 100..1000 {
            assert_eq!(3, int_log10(i), "Int log 10 failed for value {}", i);
        }
        for i in 1000..10000 {
            assert_eq!(4, int_log10(i), "Int log 10 failed for value {}", i);
        }
        for i in 10000..100000 {
            assert_eq!(5, int_log10(i), "Int log 10 failed for value {}", i);
        }
    }

    #[test]
    fn paginate_too_few() {
        let choices = ListOption::from_list(vec!["1", "2", "3"]);

        let page_size = 4usize;
        let sel = 3usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[..], page.content[..]);
        assert_eq!(3usize, page.selection);
        assert_eq!(true, page.first);
        assert_eq!(true, page.last);
        assert_eq!(3, page.total);
    }

    #[test]
    fn paginate_first_half() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 4usize;
        let sel = 2usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..4], page.content[..]);
        assert_eq!(2usize, page.selection);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_middle() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 2usize;
        let sel = 3usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[2..4], page.content[..]);
        assert_eq!(1usize, page.selection);
        assert_eq!(false, page.first);
        assert_eq!(false, page.last);
        assert_eq!(6, page.total);
    }

    #[test]
    fn paginate_lasts_half() {
        let choices = ListOption::from_list(vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 3usize;
        let sel = 5usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[3..6], page.content[..]);
        assert_eq!(2usize, page.selection);
        assert_eq!(false, page.first);
        assert_eq!(true, page.last);
        assert_eq!(6, page.total);
    }
}

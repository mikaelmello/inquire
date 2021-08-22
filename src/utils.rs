// sorry for this file

pub struct Page<'a, T> {
    pub first: bool,
    pub last: bool,
    pub content: &'a [T],
    pub selection: usize,
}

pub fn paginate<'a, T>(page_size: usize, choices: &[T], sel: usize) -> Page<T> {
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
    }
}

#[cfg(test)]
mod test {
    use crate::{option_answer::OptionAnswer, utils::paginate};

    #[test]
    fn paginate_too_few() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3"]);

        let page_size = 4usize;
        let sel = 3usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[..], page.content[..]);
        assert_eq!(3usize, page.selection);
        assert_eq!(true, page.first);
        assert_eq!(true, page.last);
    }

    #[test]
    fn paginate_first_half() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 4usize;
        let sel = 2usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..4], page.content[..]);
        assert_eq!(2usize, page.selection);
        assert_eq!(true, page.first);
        assert_eq!(false, page.last);
    }

    #[test]
    fn paginate_middle() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 2usize;
        let sel = 3usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[2..4], page.content[..]);
        assert_eq!(1usize, page.selection);
        assert_eq!(false, page.first);
        assert_eq!(false, page.last);
    }

    #[test]
    fn paginate_lasts_half() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 3usize;
        let sel = 5usize;

        let page = paginate(page_size, &choices, sel);

        assert_eq!(choices[3..6], page.content[..]);
        assert_eq!(2usize, page.selection);
        assert_eq!(false, page.first);
        assert_eq!(true, page.last);
    }
}

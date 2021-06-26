use crate::answer::OptionAnswer;

// sorry for this file

pub fn paginate(
    page_size: usize,
    choices: &[OptionAnswer],
    sel: usize,
) -> (&[OptionAnswer], usize) {
    if choices.len() < page_size {
        return (&choices[0..choices.len()], sel);
    } else if sel < page_size / 2 {
        // if we are in the first half page
        let start = 0;
        let end = page_size;
        let cursor = sel;

        return (&choices[start..end], cursor);
    } else if choices.len() - sel - 1 < page_size / 2 {
        // if we are in the last half page
        let start = choices.len() - page_size;
        let end = choices.len();
        let cursor = sel - start;
        return (&choices[start..end], cursor);
    } else {
        // somewhere in the middle
        let above = page_size / 2;
        let below = page_size - above;

        let cursor = page_size / 2;
        let start = sel - above;
        let end = sel + below;

        return (&choices[start..end], cursor);
    }
}

#[cfg(test)]
mod test {
    use crate::utils::paginate;

    use super::OptionAnswer;

    #[test]
    fn paginate_too_few() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3"]);

        let page_size = 4usize;
        let sel = 3usize;

        let (page, idx) = paginate(page_size, &choices, sel);

        assert_eq!(choices[..], page[..]);
        assert_eq!(3usize, idx);
    }

    #[test]
    fn paginate_first_half() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 4usize;
        let sel = 2usize;

        let (page, idx) = paginate(page_size, &choices, sel);

        assert_eq!(choices[0..4], page[..]);
        assert_eq!(2usize, idx);
    }

    #[test]
    fn paginate_middle() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 2usize;
        let sel = 3usize;

        let (page, idx) = paginate(page_size, &choices, sel);

        assert_eq!(choices[2..4], page[..]);
        assert_eq!(1usize, idx);
    }

    #[test]
    fn paginate_lasts_half() {
        let choices = OptionAnswer::from_str_list(&vec!["1", "2", "3", "4", "5", "6"]);

        let page_size = 3usize;
        let sel = 5usize;

        let (page, idx) = paginate(page_size, &choices, sel);

        assert_eq!(choices[3..6], page[..]);
        assert_eq!(2usize, idx);
    }
}

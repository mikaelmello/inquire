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

#[cfg(target_pointer_width = "16")]
const USIZE_BYTES: usize = 2;
#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;
#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;
const LO: usize = ::std::usize::MAX / 255;
const HI: usize = LO * 128;
const REP_NEWLINE: usize = b'\n' as usize * LO;

const EVERY_OTHER_BYTE_LO: usize = 0x0001000100010001;
const EVERY_OTHER_BYTE: usize = EVERY_OTHER_BYTE_LO * 0xFF;

// https://github.com/llogiq/newlinebench
pub fn count_newlines_hyperscreaming(s: &str) -> usize {
    unsafe {
        let text = s.as_bytes();
        let mut ptr = text.as_ptr();
        let mut end = ptr.offset(text.len() as isize);

        let mut count = 0;

        // Align start
        while (ptr as usize) & (USIZE_BYTES - 1) != 0 {
            if ptr == end {
                return count;
            }
            count += (*ptr == b'\n') as usize;
            ptr = ptr.offset(1);
        }

        // Align end
        while (end as usize) & (USIZE_BYTES - 1) != 0 {
            end = end.offset(-1);
            count += (*end == b'\n') as usize;
        }
        if ptr == end {
            return count;
        }

        // Read in aligned blocks
        let mut ptr = ptr as *const usize;
        let end = end as *const usize;

        unsafe fn next(ptr: &mut *const usize) -> usize {
            let ret = **ptr;
            *ptr = ptr.offset(1);
            ret
        }

        fn mask_zero(x: usize) -> usize {
            (((x ^ REP_NEWLINE).wrapping_sub(LO)) & !x & HI) >> 7
        }

        unsafe fn next_4(ptr: &mut *const usize) -> [usize; 4] {
            let x = [next(ptr), next(ptr), next(ptr), next(ptr)];
            [
                mask_zero(x[0]),
                mask_zero(x[1]),
                mask_zero(x[2]),
                mask_zero(x[3]),
            ]
        }

        fn reduce_counts(counts: usize) -> usize {
            let pair_sum = (counts & EVERY_OTHER_BYTE) + ((counts >> 8) & EVERY_OTHER_BYTE);
            pair_sum.wrapping_mul(EVERY_OTHER_BYTE_LO) >> ((USIZE_BYTES - 2) * 8)
        }

        fn arr_add(xs: [usize; 4], ys: [usize; 4]) -> [usize; 4] {
            [xs[0] + ys[0], xs[1] + ys[1], xs[2] + ys[2], xs[3] + ys[3]]
        }

        // 8kB
        while ptr.offset(4 * 255) <= end {
            let mut counts = [0, 0, 0, 0];
            for _ in 0..255 {
                counts = arr_add(counts, next_4(&mut ptr));
            }
            count += reduce_counts(counts[0]);
            count += reduce_counts(counts[1]);
            count += reduce_counts(counts[2]);
            count += reduce_counts(counts[3]);
        }

        // 1kB
        while ptr.offset(4 * 32) <= end {
            let mut counts = [0, 0, 0, 0];
            for _ in 0..32 {
                counts = arr_add(counts, next_4(&mut ptr));
            }
            count += reduce_counts(counts[0] + counts[1] + counts[2] + counts[3]);
        }

        // 64B
        let mut counts = [0, 0, 0, 0];
        while ptr.offset(4 * 2) <= end {
            for _ in 0..2 {
                counts = arr_add(counts, next_4(&mut ptr));
            }
        }
        count += reduce_counts(counts[0] + counts[1] + counts[2] + counts[3]);

        // 8B
        let mut counts = 0;
        while ptr < end {
            counts += mask_zero(next(&mut ptr));
        }
        count += reduce_counts(counts);

        count
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

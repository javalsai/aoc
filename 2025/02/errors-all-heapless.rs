//! Based from from ErrorNoInternet's <https://github.com/ErrorNoInternet/>. Not public code but
//! got permission to tweak it to perfection.
//!
//! Ugh, I guess all rights reserved to him unless he indicates otherwise explicitly.
//!
//! Insane speed tho, the magic of doing the math.

struct StackLinkedList<'a, T> {
    next: Option<&'a Self>,
    data: T,
}
impl<'a, T> IntoIterator for &'a StackLinkedList<'a, T> {
    type Item = &'a T;
    type IntoIter = SLLIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SLLIter { node: Some(self) }
    }
}

use StackLinkedList as SLL;

pub struct SLLIter<'a, T> {
    node: Option<&'a SLL<'a, T>>,
}
impl<'a, T> Iterator for SLLIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = node.next;
            Some(&node.data)
        } else {
            None
        }
    }
}

#[unsafe(no_mangle)]
fn challenge_usize_duple(b: &[u8]) -> (usize, usize) {
    let s = unsafe { str::from_utf8_unchecked(b) };
    let mut parsing_iter = s.trim_end().split(",").map(|range| {
        let (left, right) = range.trim().split_once("-").unwrap();
        (left.parse().unwrap(), right.parse().unwrap())
    });

    collect_to_stack_ll_and_call(&mut parsing_iter, |ranges| {
        let a = sum(ranges, &[2, 4, 6, 8, 10], &[1, 2, 3, 4, 5]);
        let b = sum(ranges, &[3, 5, 6, 7, 9, 10], &[1, 1, 2, 1, 3, 2])
            - sum(ranges, &[6, 10], &[1, 1]);

        (a, a + b)
    }).unwrap()
}

fn collect_to_stack_ll_and_call<R>(
    from: &mut impl Iterator<Item = (usize, usize)>,
    f: impl FnOnce(&SLL<(usize, usize)>) -> R,
) -> Option<R> {
    fn _rec<R>(
        from: &mut impl Iterator<Item = (usize, usize)>,
        link_to: &SLL<(usize, usize)>,
        f: impl FnOnce(&SLL<(usize, usize)>) -> R,
    ) -> R {
        if let Some(e) = from.next() {
            let node = StackLinkedList {
                next: Some(link_to),
                data: e,
            };
            _rec(from, &node, f)
        } else {
            (f)(link_to)
        }
    }

    if let Some(e) = from.next() {
        let node = StackLinkedList {
            next: None,
            data: e,
        };
        Some(_rec(from, &node, f))
    } else {
        None
    }
}

fn sum<'a>(ranges: impl IntoIterator<Item = &'a (usize, usize)> + Copy, digits: &[u32], lengths: &[u32]) -> usize {
    let mut result = 0;

    for (d, l) in digits.iter().zip(lengths) {
        let repetitions = d / l;
        let power = 10_usize.pow(*l);
        let mut step = 0;
        for _ in 0..repetitions {
            step = step * power + 1;
        }
        let invalid_start = step * (power / 10);
        let invalid_end = step * (power - 1);

        for &(start, end) in ranges.into_iter() {
            let lower = start.next_multiple_of(step).max(invalid_start);
            let upper = end.min(invalid_end);
            if lower <= upper {
                let n = (upper - lower) / step;
                let m = n * (n + 1) / 2;
                result += lower * (n + 1) + step * m;
            }
        }
    }

    result
}

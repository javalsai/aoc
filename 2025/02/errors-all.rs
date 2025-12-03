//! Based from from ErrorNoInternet's <https://github.com/ErrorNoInternet/>. Not public code but
//! got permission to tweak it to perfection.
//!
//! Ugh, I guess all rights reserved to him unless he indicates otherwise explicitly.
//!
//! Insane speed tho, the magic of doing the math.

#[unsafe(no_mangle)]
fn challenge_usize_duple(b: &[u8]) -> (usize, usize) {
    let s = unsafe { str::from_utf8_unchecked(b) };
    let ranges = ranges(s.trim_end());
    let ranges: Vec<_> = ranges.collect();

    let a = sum(&ranges, &[2, 4, 6, 8, 10], &[1, 2, 3, 4, 5]);
    let b =
        sum(&ranges, &[3, 5, 6, 7, 9, 10], &[1, 1, 2, 1, 3, 2]) - sum(&ranges, &[6, 10], &[1, 1]);

    (a, a + b)
}

fn ranges(input: &str) -> impl Iterator<Item = (usize, usize)> + Clone {
    input.split(",").map(|range| {
        let (left, right) = range.trim().split_once("-").unwrap();
        (left.parse().unwrap(), right.parse().unwrap())
    })
}

fn sum(ranges: &[(usize, usize)], digits: &[u32], lengths: &[u32]) -> usize {
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

        for &(start, end) in ranges {
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

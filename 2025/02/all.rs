#[unsafe(no_mangle)]
fn challenge_usize_duple(b: &[u8]) -> (usize, usize) {
    let mut total1 = 0;
    let mut total2 = 0;
    let s = unsafe { str::from_utf8_unchecked(b) };

    let ranges = s.trim_end().split(',');
    for range in ranges {
        let (start, end) = range.split_once('-').unwrap();
        let (start, end): (usize, usize) = (start.parse().unwrap(), end.parse().unwrap());

        for id in start..=end {
            let mut buf = [0; _];
            let id_s = &fast_to_string(id, &mut buf);

            if is_repeating1(id_s) {
                total1 += id;
            }
            if is_repeating2(id_s) {
                total2 += id;
            }
        }
    }

    (total1, total2)
}

const MAX_USIZE_LEN: usize = const { usize::MAX.ilog10() + 1 } as usize;
/// Output is reversed btw
fn fast_to_string(mut n: usize, into: &mut [u8; MAX_USIZE_LEN]) -> &mut [u8] {
    let mut len = 0;
    while n != 0 {
        into[len] = (n % 10) as u8 + b'0';
        n /= 10;
        len += 1;
    }

    &mut into[0..len]
}

fn is_repeating1(s: &[u8]) -> bool {
    if !s.len().is_multiple_of(2) {
        return false;
    }

    let mid = s.len() / 2;
    let (left, right) = s.split_at(mid);
    left == right
}

fn is_repeating2(s: &[u8]) -> bool {
    for sublen in 0..(s.len() / 2) {
        let sublen = sublen + 1;
        if !s.len().is_multiple_of(sublen) {
            continue;
        }
        let pref = &s[0..sublen];

        let is_repeating = s[sublen..].chunks_exact(sublen).all(|c| c == pref);
        if is_repeating {
            return true;
        }
    }

    false
}

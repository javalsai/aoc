pub type RangeU = usize;

#[unsafe(no_mangle)]
extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let s = unsafe { str::from_utf8_unchecked(buf) };
    let mut lines = s.lines();

    let mut ranges: Vec<(RangeU, RangeU)> = Vec::new();

    for range in &mut lines {
        if range.is_empty() {
            break;
        }

        let (l, r) = range.split_once('-').unwrap();
        let (l, r) = (l.parse().unwrap(), r.parse().unwrap());

        ranges.push((l, r));
    }

    ranges.sort_by(|a, b| a.0.cmp(&b.0));
    ranges
        .into_iter()
        .fold((0, 0), |(count, rightmost), (l, r)| {
            // dbg!((rightmost, r));
            if rightmost >= r {
                (count, rightmost)
            } else if rightmost >= l {
                let overlap = rightmost - l + 1;
                (count + r - l - overlap + 1, rightmost.max(r))
            } else {
                (count + r - l + 1, r)
            }
        }).0
}

pub type RangeU = usize;

#[unsafe(no_mangle)]
extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let s = unsafe { str::from_utf8_unchecked(buf) };
    let mut lines = s.lines();

    let mut count = 0;

    let mut ranges: Vec<(RangeU, RangeU)> = Vec::new();

    for range in &mut lines {
        if range.is_empty() {
            break;
        }

        let (l, r) = range.split_once('-').unwrap();
        let (l, r) = (l.parse().unwrap(), r.parse().unwrap());

        // overlapping here means overlapping or adyacent
        count += r - l + 1;
        let overlapping: Option<&mut (RangeU, RangeU)> = None;
        for (r_l, r_r) in ranges.iter_mut() {
            if l <= *r_r && r >= *r_r {
                *r_r = r;
                //
            }
            // (|(l, r)| (l..=r).contains(&fresh))
        }
        for fresh in l..=r {
            println!("{fresh:?}");
            if ranges.iter().any(|&(l, r)| (l..=r).contains(&fresh)) {
                count -= 1;
            }
        }
        ranges.push((l, r));
    }

    count
}

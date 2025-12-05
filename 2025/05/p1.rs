use std::time::{Duration, Instant};

#[unsafe(no_mangle)]
pub static mut TIMERS: [(&str, Duration); TIMERS_LEN] = [
    ("after-parse", Duration::ZERO),
    ("sort", Duration::ZERO),
    ("count-ids", Duration::ZERO),
];
#[unsafe(no_mangle)]
pub static TIMERS_LEN: usize = 3;

pub type RangeU = u128;

#[unsafe(no_mangle)]
extern "Rust" fn challenge_t_usize(buf: &[u8], t: &Instant) -> usize {
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

        ranges.push((l, r));
    }

    unsafe { TIMERS[0].1 = t.elapsed() };

    ranges.sort_by(|a, b| a.0.cmp(&b.0));

    unsafe { TIMERS[1].1 = t.elapsed() };

    for id in lines {
        let id: RangeU = id.parse().unwrap();

        if ranges.iter().any(|&(l, r)| (l..=r).contains(&id)) {
            count += 1;
        }
    }

    unsafe { TIMERS[2].1 = t.elapsed() };

    count
}

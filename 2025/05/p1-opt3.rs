use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

pub type RangeU = u128;
pub type NestingCountT = i16;

#[unsafe(no_mangle)]
pub static mut TIMERS: [(&str, Duration); TIMERS_LEN] = [
    ("after-parse", Duration::ZERO),
    ("flatten-vec", Duration::ZERO),
    ("count-ids", Duration::ZERO),
];
#[unsafe(no_mangle)]
pub static TIMERS_LEN: usize = 3;

#[unsafe(no_mangle)]
extern "Rust" fn challenge_t_usize(buf: &[u8], t: &Instant) -> usize {
    let mut count = 0;
    let s = unsafe { str::from_utf8_unchecked(buf) };
    let mut lines = s.lines();

    let mut bmap = BTreeMap::<_, NestingCountT>::new();

    (&mut lines)
        .take_while(|ln| !ln.is_empty())
        .for_each(|range| {
            let (l, r) = range.split_once('-').unwrap();
            let (l, r) = (l.parse().unwrap(), r.parse().unwrap());

            bmap.entry(l).and_modify(|k| *k += 1).or_insert(1);
            bmap.entry(r).and_modify(|k| *k -= 1).or_insert(-1);
        });

    unsafe { TIMERS[0].1 = t.elapsed() };

    let mut arr = Vec::with_capacity(size_of::<bool>() * bmap.len());
    // I could directly search on the btree, but I feel flattening at once and then searching can
    // speed up things bcs continuity indirection and stuff. It also makes it easier to iterate at
    // once. This would be O(n) and the latter access in O(log n) (O(n log n) bcs it iterates for
    // each id)
    let mut last_pushed = false;
    let mut nested_ran_acc = 0;
    for (idx, nesting) in bmap {
        nested_ran_acc += nesting;
        if last_pushed != (nested_ran_acc > 0) {
            last_pushed = !last_pushed;
            arr.push((idx, last_pushed));
        }
    }

    unsafe { TIMERS[1].1 = t.elapsed() };

    let all_ranges_bounds = (arr[0].0, arr[arr.len() - 1].0);
    for id in lines {
        let id: RangeU = id.parse().unwrap();
        if id < all_ranges_bounds.0 || id > all_ranges_bounds.1 {
            continue;
        }

        // let idx = arr.binary_search_by_key(&id, |p| p.0).err(|i| i);
        // let is_in_any_range = arr.get(idx).map(|p| p.1).unwrap();
        let is_in_any_range = binary_search_inclusive_or_lower_bound(&arr, id);

        if is_in_any_range {
            count += 1
        }
    }

    unsafe { TIMERS[2].1 = t.elapsed() };

    count
}

fn binary_search_inclusive_or_lower_bound(arr: &[(RangeU, bool)], id: RangeU) -> bool {
    let all_ranges_bounds = (arr[0].0, arr[arr.len() - 1].0);

    if id < all_ranges_bounds.0 || id > all_ranges_bounds.1 {
        return false;
    }

    match arr.binary_search_by_key(&id, |p| p.0) {
        Ok(_) => true, // the rising edge is true and if falling edge its false but included so
        // should get truthed. In general if its found it was an edge so return true
        Err(idx) => arr[idx - 1].1,
    }
}

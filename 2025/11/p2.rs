#![feature(slice_split_once)]

use std::{collections::HashMap, hash::Hash};

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let conns = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(|ln| {
            let (left, right) = ln.split_once(|&b| b == b':').unwrap();
            let conns = unsafe { str::from_utf8_unchecked(right) }
                .trim()
                .split(' ')
                .collect::<Vec<_>>();

            (unsafe { str::from_utf8_unchecked(left) }, conns)
        })
        .collect::<HashMap<_, _>>();

    let mut hoist = HashMap::new();
    count_paths(&mut hoist, &conns, "svr", false, false)
}

fn count_paths<'a>(
    hoist: &mut HashMap<(&'a str, bool, bool), usize>,
    set: &HashMap<&str, Vec<&'a str>>,
    from: &'a str,
    found_fft: bool,
    found_dac: bool,
) -> usize {
    let mut count = 0;
    if let Some(&hoisted) = hoist.get(&(from, found_fft, found_dac)) {
        return hoisted;
    };

    for &subk in set.get(from).unwrap() {
        if subk == "out" {
            if found_dac && found_fft {
                count += 1;
            }
        } else {
            let found_fft = found_fft || subk == "fft";
            let found_dac = found_dac || subk == "dac";

            let val = count_paths(hoist, set, subk, found_fft, found_dac);

            hoist.insert((subk, found_fft, found_dac), val);

            count += val;
        }
    }

    count
}

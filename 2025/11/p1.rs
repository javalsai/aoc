#![feature(slice_split_once)]

use std::collections::HashMap;

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
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

    println!("{conns:#?}");

    count_paths(&conns, "you")
}

fn count_paths(set: &HashMap<&str, Vec<&str>>, from: &str) -> usize {
    let mut count = 0;

    for &subk in set.get(from).unwrap() {
        if subk == "out" {
            count += 1;
        } else {
            count += count_paths(set, subk);
        }
    }

    count
}

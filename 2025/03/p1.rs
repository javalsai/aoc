use std::cmp::Ordering;

fn cmp_with_eq_as_gt<T: Ord>(a: &T, b: &T) -> Ordering {
    let order = a.cmp(b);
    if order == Ordering::Equal {
        Ordering::Greater
    } else {
        order
    }
}

#[unsafe(no_mangle)]
fn challenge_usize(buf: &[u8]) -> usize {
    let mut total_joltage = 0;

    let s = unsafe { str::from_utf8_unchecked(buf) };
    for array in s.trim().split('\n') {
        let bats = array.as_bytes();
        let bats_but_last = &bats[0..(bats.len() - 1)];

        let first_dig = bats_but_last
            .iter()
            .enumerate()
            .max_by(|a, b| cmp_with_eq_as_gt(a.1, b.1))
            .unwrap();
        let last_dig = bats[(first_dig.0 + 1)..].iter().max().unwrap();

        // println!("-->> {first_dig:?} {last_dig}");
        let arr_max_jolts = ((first_dig.1 - b'0') * 10) + (last_dig - b'0');
        total_joltage += Into::<usize>::into(arr_max_jolts);
    }

    total_joltage
}

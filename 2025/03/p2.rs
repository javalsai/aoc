use std::cmp::Ordering;

fn cmp_with_eq_as_gt<T: Ord>(a: &T, b: &T) -> Ordering {
    let order = a.cmp(b);
    if order == Ordering::Equal {
        Ordering::Greater
    } else {
        order
    }
}

const DIGS: usize = 12;

#[unsafe(no_mangle)]
fn challenge_usize(buf: &[u8]) -> usize {
    let mut total_joltage = 0;

    let s = unsafe { str::from_utf8_unchecked(buf) };
    for array in s.trim().split('\n') {
        let bats = array.as_bytes();

        let mut digs = [0u8; DIGS];
        digs.iter_mut()
            .enumerate()
            .fold(0, |max_left, (i, dig): (_, &mut u8)| {
                let max_bat = bats[max_left..(bats.len() - DIGS + i + 1)]
                    .iter()
                    .enumerate()
                    .max_by(|a, b| cmp_with_eq_as_gt(a.1, b.1))
                    .unwrap();

                *dig = *max_bat.1;

                max_left + max_bat.0 + 1
            });

        let n = digs
            .iter()
            .map(|d| d - b'0')
            .fold(0_usize, |left, r| left * 10 + Into::<usize>::into(r));

        // println!("-->> {digs:?} {n:?}",);
        total_joltage += n;
    }

    total_joltage
}

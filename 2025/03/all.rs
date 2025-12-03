use std::cmp::Ordering;

fn cmp_with_eq_as_gt<T: Ord>(a: &T, b: &T) -> Ordering {
    let order = a.cmp(b);
    if order == Ordering::Equal {
        Ordering::Greater
    } else {
        order
    }
}

fn p1(bats: [u8; DIGS]) -> usize {
    let bats_but_last = &bats[0..(bats.len() - 1)];

    let first_dig = unsafe {
        bats_but_last
            .iter()
            .enumerate()
            .max_by(|a, b| cmp_with_eq_as_gt(a.1, b.1))
            .unwrap_unchecked()
    };
    let last_dig = unsafe { bats[(first_dig.0 + 1)..].iter().max().unwrap_unchecked() };

    let arr_max_jolts = ((first_dig.1 - b'0') * 10) + (last_dig - b'0');

    Into::<usize>::into(arr_max_jolts)
}

const DIGS: usize = 12;

#[unsafe(no_mangle)]
fn challenge_usize_duple(buf: &[u8]) -> (usize, usize) {
    let mut total_joltage1 = 0;
    let mut total_joltage2 = 0;

    let s = unsafe { str::from_utf8_unchecked(buf) };
    for array in s.trim().split('\n') {
        let bats = array.as_bytes();

        let mut digs = [0u8; DIGS];
        digs.iter_mut()
            .enumerate()
            .fold(0, |max_left, (i, dig): (_, &mut u8)| {
                let max_bat = unsafe {
                    bats[max_left..(bats.len() - DIGS + i + 1)]
                        .iter()
                        .enumerate()
                        .max_by(|a, b| cmp_with_eq_as_gt(a.1, b.1))
                        .unwrap_unchecked()
                };

                *dig = *max_bat.1;
                max_left + max_bat.0 + 1
            });

        let n = digs
            .iter()
            .map(|d| d - b'0')
            .fold(0_usize, |left, r| left * 10 + Into::<usize>::into(r));

        total_joltage1 += p1(digs);
        total_joltage2 += n;
    }

    (total_joltage1, total_joltage2)
}

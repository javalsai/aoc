/*
rustc --edition 2024 sped.rs -O -Copt-level=3 -Cstrip=symbols -Cdebuginfo=0 -Cdebug-assertions=off -Coverflow-checks=false -Cpanic=abort -Ctarget-cpu=native -Ccodegen-units=1
*/

use std::{
    cmp::Ordering,
    fs::File,
    io::Read,
    mem::MaybeUninit,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering::Relaxed},
    },
    thread,
    time::Instant,
};

// fuck it, assume sizes and multithread
fn main() {
    let timer = Instant::now();

    let total_joltage1 = AtomicUsize::new(0);
    let total_joltage2 = AtomicUsize::new(0);

    let f = Mutex::new(File::open("2025-3.txt").unwrap());

    thread::scope(|s| {
        let c = || {
            #[allow(clippy::uninit_assumed_init, invalid_value)]
            let mut lnbuf: [u8; 50100] = unsafe { MaybeUninit::uninit().assume_init() };

            while {
                let mut flock = f.lock().unwrap();
                let flock_read = flock.read_exact(&mut lnbuf);
                flock_read.is_ok()
            } {
                for buf in lnbuf.chunks_exact(501) {
                    let bats = &buf[0..=500];

                    #[allow(clippy::uninit_assumed_init, invalid_value)]
                    let mut digs: [u8; DIGS] = unsafe { MaybeUninit::uninit().assume_init() };
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

                    total_joltage1.fetch_add(p1(digs), Relaxed);
                    total_joltage2.fetch_add(n, Relaxed);
                }
            }
        };

        for _ in 0..16 {
            s.spawn(c);
        }
    });

    println!(
        "p1 {}, p2 {}\n in {:#?}",
        total_joltage1.load(std::sync::atomic::Ordering::SeqCst),
        total_joltage2.load(std::sync::atomic::Ordering::SeqCst),
        timer.elapsed()
    );
}

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

// fn main() {
//     let timer = Instant::now();

//     let mut total_joltage1 = 0;
//     let mut total_joltage2 = 0;

//     let f = File::open("2025-3.txt").unwrap();
//     let f = BufReader::new(f);

//     for ln in f.split(b'\n') {
//         let ln = ln.unwrap();
//         let bats = &ln;

//         let mut digs = [0u8; DIGS];
//         digs.iter_mut()
//             .enumerate()
//             .fold(0, |max_left, (i, dig): (_, &mut u8)| {
//                 let max_bat = unsafe {
//                     bats[max_left..(bats.len() - DIGS + i + 1)]
//                         .iter()
//                         .enumerate()
//                         .max_by(|a, b| cmp_with_eq_as_gt(a.1, b.1))
//                         .unwrap_unchecked()
//                 };

//                 *dig = *max_bat.1;
//                 max_left + max_bat.0 + 1
//             });

//         let n = digs
//             .iter()
//             .map(|d| d - b'0')
//             .fold(0_usize, |left, r| left * 10 + Into::<usize>::into(r));

//         total_joltage1 += p1(digs);
//         total_joltage2 += n;
//     }

//     println!(
//         "p1 {total_joltage1}, p2 {total_joltage2}\n in {:#?}",
//         timer.elapsed()
//     );
// }

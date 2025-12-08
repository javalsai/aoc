#![feature(linked_list_cursors)]
use std::{collections::HashSet, ptr::from_mut};

pub type Coords = (usize, usize, usize);

const N: usize = 1000;

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let ln_iter = buf[..(buf.len() - 1)].split(|&b| b == b'\n');

    let mut pairs = {
        let mut uninit = Box::<[(usize, (Coords, Coords))]>::new_uninit_slice(N);
        uninit.iter_mut().for_each(|f| {
            f.write((usize::MAX, ((0, 0, 0), (0, 0, 0))));
        });
        unsafe { uninit.assume_init() }
    };

    let mut coords = Vec::new();
    for ln in ln_iter {
        let this_box = parse_jbox_ln(ln);
        coords.push(this_box);
    }

    for (i, &a) in coords.iter().enumerate() {
        for &b in &coords[(i + 1)..] {
            // println!("{:?}", pairs.iter().map(|f| f.0).collect::<Vec<_>>());
            maybe_insert_in(a, b, pairs.as_mut());
        }
    }

    // let mut maxs = [0, 0, 0];

    let mut seen_ones: HashSet<Coords> = HashSet::new();
    let mut groups: Vec<HashSet<Coords>> = Vec::new();
    while let Some((i, &(_, (a, b)))) = pairs
        .iter()
        .enumerate()
        .find(|(_, (_, (a, b)))| !seen_ones.contains(a) && !seen_ones.contains(b))
    {
        let mut this_max = HashSet::new();
        this_max.insert(a);
        this_max.insert(b);

        let pairs_iter = &pairs[(i + 1)..];

        let mut inserted_one = true;
        while inserted_one {
            inserted_one = false;

            for &(_, (a, b)) in pairs_iter {
                if this_max.contains(&a) || this_max.contains(&b) {
                    #[allow(clippy::collapsible_if)]
                    if this_max.insert(a) || this_max.insert(b) {
                        inserted_one = true;
                    }
                }
            }
        }

        seen_ones.extend(this_max.iter());
        groups.push(this_max);
    }

    // println!("{pairs:#?} -> {maxs:?}");

    groups.sort_by_key(|b| std::cmp::Reverse(b.len()));
    groups[0..=2].iter().map(|f| f.len()).product()
    // maxs.iter().product()
}

// fn shift_insert_at<T>(what: T, mut at: usize, slice: &mut [T]) {
//     let mut saved = what;

//     while at < slice.len() {
//         std::mem::swap(&mut slice[at], &mut saved);
//         at += 1;
//     }
// }

fn shift_insert_at<T>(what: T, at: usize, slice: &mut [T]) {
    if at >= slice.len() {
        return;
    }
    let tail = &mut slice[at..];
    tail.rotate_right(1);
    tail[0] = what;
}

fn maybe_insert_in(a: Coords, b: Coords, into: &mut [(usize, (Coords, Coords))]) {
    let dist = distance(a, b);
    let mut res = into
        .binary_search_by_key(&dist, |&(dist2, _)| dist2)
        .unwrap_or_else(|e| e);

    if res < into.len() {
        while into[res].0 > dist && res > 0 {
            res -= 1;
        }

        while into[res].0 < dist {
            res += 1;
        }

        shift_insert_at((dist, (a, b)), res, into);
    }
}

fn threemax_insert(max: &mut (usize, usize, usize), n: usize) {
    if n > max.0 {
        max.2 = max.1;
        max.1 = max.0;
        max.0 = n;
    } else if n > max.1 {
        max.2 = max.1;
        max.1 = n;
    } else if n > max.2 {
        max.2 = n
    }
}

fn distance(b1: Coords, b2: Coords) -> usize {
    let a = b1.0 - b2.0;
    let b = b1.1 - b2.1;
    let c = b1.2 - b2.2;

    (a * a + b * b + c * c).isqrt()
}

fn merge_shortest_pair(from: &mut Vec<HashSet<Coords>>) {
    let mut min = None;

    for (i, circ) in unsafe { &mut *std::ptr::from_mut(from) }
        .iter_mut()
        .enumerate()
    {
        let raw_circ = from_mut(circ);

        for &coor in circ.iter() {
            let rest_slice = &mut from[(i + 1)..];
            for other_circ in rest_slice.iter_mut() {
                let raw_other_circ = from_mut(other_circ);

                for &coor2 in other_circ.iter() {
                    let dist = distance(coor, coor2);

                    match min {
                        Some((min_dist, _, _, _)) => {
                            if min_dist > dist && dist != 0 {
                                min = Some((dist, raw_circ, raw_other_circ, coor2))
                            }
                        }
                        None => min = Some((dist, raw_circ, raw_other_circ, coor2)),
                    }
                }
            }
        }
    }

    let Some((_, circ1, circ2, coor)) = min else {
        panic!("no pair found?");
    };

    let (circ1, circ2) = (unsafe { &mut *circ1 }, unsafe { &mut *circ2 });
    circ1.insert(coor);
    circ2.remove(&coor);
}

fn parse_jbox_ln(ln: &[u8]) -> Coords {
    let mut iter = ln.split(|&b| b == b',').map(|num_bytes| {
        num_bytes
            .iter()
            .fold(0, |acc, b| acc * 10 + ((*b - b'0') as usize))
    });

    (
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap(),
    )
}

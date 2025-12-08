#![feature(linked_list_cursors)]
use std::{collections::HashSet, ptr::from_mut};

pub type Coords = (usize, usize, usize);

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let ln_iter = buf[..(buf.len() - 1)].split(|&b| b == b'\n');

    let mut circuits: Vec<HashSet<Coords>> = Vec::new();

    for ln in ln_iter {
        let this_box = parse_jbox_ln(ln);
        circuits.push(HashSet::new());
        circuits.last_mut().unwrap().insert(this_box);
    }

    let mut prev_mul = None;
    while let Some(this_mul) = merge_shortest_pair(&mut circuits) {
        prev_mul = Some(this_mul);
    }

    prev_mul.expect("not a single merge was done")
}

fn distance(b1: Coords, b2: Coords) -> usize {
    let a = b1.0 - b2.0;
    let b = b1.1 - b2.1;
    let c = b1.2 - b2.2;

    (a * a + b * b + c * c).isqrt()
}

fn merge_shortest_pair(from: &mut Vec<HashSet<Coords>>) -> Option<usize> {
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
                        Some((min_dist, _, _, _, _)) => {
                            if min_dist > dist && dist != 0 {
                                min =
                                    Some((dist, raw_circ, raw_other_circ, coor2, coor.0 * coor2.0));
                            }
                        }
                        None => {
                            min = Some((dist, raw_circ, raw_other_circ, coor2, coor.0 * coor2.0));
                        }
                    }
                }
            }
        }
    }

    if let Some((_, circ1, circ2, coor, that_mul)) = min {
        let (circ1, circ2) = (unsafe { &mut *circ1 }, unsafe { &mut *circ2 });
        circ1.insert(coor);
        circ2.remove(&coor);
        Some(that_mul)
    } else {
        None
    }
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

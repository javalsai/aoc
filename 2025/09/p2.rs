#![feature(iter_map_windows)]
//! Half of the puzzle seems like a fancy way to say that the puzzle corners have to be within any
//! other rectangle or adyacent (as in same x or y as any other point) to other corners.
//!
//! I think even adyacent is within the "other rectangle thing", because

use std::{collections::BinaryHeap, ops::Range};

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let coords = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(parse_ln)
        .collect::<Vec<_>>();

    // assuming each coord is contiguous to the prev one
    let edges = coords
        .iter()
        .cloned()
        .chain([coords[0]])
        .map_windows(|&[a, b]| (a.0.abs_diff(b.0) + a.1.abs_diff(b.1), (a, b)))
        .collect::<BinaryHeap<_>>();

    //     edges.sort_by(|(a1, a2), (b1, b2)| {
    //         (a1.0.abs_diff(a2.0) + a1.1.abs_diff(a2.1))
    //             .cmp(&(b1.0.abs_diff(b2.0) + b1.1.abs_diff(b2.1)))
    //     });

    let mut max_area = 0;
    for (i, coor1) in coords.iter().enumerate() {
        for coor2 in coords.iter().skip(i + 1) {
            let dx = coor1.0.abs_diff(coor2.0) + 1;
            let dy = coor1.1.abs_diff(coor2.1) + 1;
            let area = dx * dy;

            if area > max_area
                && is_really_contained((*coor1, *coor2), edges.iter().map(|&(_, v)| v))
            {
                max_area = area;
            }
        }
    }

    max_area
}

/// If any bouding vertex is well within (not sitting on a rectangle's edge), the rectangle is not
/// well contained
fn is_really_contained(
    (rect0, rect1): ((usize, usize), (usize, usize)),
    edges: impl Iterator<Item = ((usize, usize), (usize, usize))>,
) -> bool {
    let (rect0, rect1) = (
        (rect0.0.min(rect1.0), rect0.1.min(rect1.1)),
        (rect0.0.max(rect1.0), rect0.1.max(rect1.1)),
    );

    let xran = (rect0.0 + 1)..(rect1.0);
    let yran = (rect0.1 + 1)..(rect1.1);

    // Optimization, no need to check each range's point
    for (edge1, edge2) in edges {
        if edge1.0 == edge2.0
            && xran.contains(&edge1.0)
            && rangeoverlap(&mkrange(edge1.1, edge2.1), &yran)
        {
            return false;
        }

        if edge1.1 == edge2.1
            && yran.contains(&edge1.1)
            && rangeoverlap(&mkrange(edge1.0, edge2.0), &xran)
        {
            return false;
        }
    }

    true
}

fn mkrange<T: Ord + Copy>(a: T, b: T) -> Range<T> {
    a.min(b)..a.max(b)
}

fn rangeoverlap<T: Ord>(a: &Range<T>, b: &Range<T>) -> bool {
    if a.end <= b.start || a.start >= b.end {
        return false;
    }

    true
}

fn parse_ln(ln: &[u8]) -> (usize, usize) {
    let mut iter = ln.split(|&b| b == b',').map(|slice| {
        slice
            .iter()
            .fold(0, |acc, b| acc * 10 + (b - b'0') as usize)
    });

    (iter.next().unwrap(), iter.next().unwrap())
}

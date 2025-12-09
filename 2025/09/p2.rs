#![feature(iter_map_windows)]
//! Half of the puzzle seems like a fancy way to say that the puzzle corners have to be within any
//! other rectangle or adyacent (as in same x or y as any other point) to other corners.
//!
//! I think even adyacent is within the "other rectangle thing", because

#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    // I do see how to make this in idk if O(n) or O(nlogn), but ima O(n^2) just at first
    let coords = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(parse_ln)
        .collect::<Vec<_>>();

    // assuming each coord is contiguous to the prev one
    let edges = coords
        .iter()
        .cloned()
        .chain([coords[0]])
        .map_windows(|&[a, b]| (a, b))
        .collect::<Vec<_>>();

    let mut area = 0;
    for coor1 in &coords {
        for coor2 in &coords {
            let dx = coor1.0.abs_diff(coor2.0) + 1;
            let dy = coor1.1.abs_diff(coor2.1) + 1;
            let this_area = dx * dy;

            // println!("{coor1:?}, {coor2:?}");
            if is_really_contained((*coor1, *coor2), &edges) {
                area = area.max(this_area);
            }

            // println!(
            //     "{},{} {},{} | {this_area}",
            //     coor1.0, coor1.1, coor2.0, coor2.1
            // );
        }
    }

    area
}

/// If any bouding vertex is well within (not sitting on a rectangle's edge), the rectangle is not
/// well contained
fn is_really_contained(
    (rect0, rect1): ((usize, usize), (usize, usize)),
    edges: &[((usize, usize), (usize, usize))],
) -> bool {
    let (rect0, rect1) = (
        (rect0.0.min(rect1.0), rect0.1.min(rect1.1)),
        (rect0.0.max(rect1.0), rect0.1.max(rect1.1)),
    );

    let xran = (rect0.0 + 1)..=(rect1.0 - 1);
    let yran = (rect0.1 + 1)..=(rect1.1 - 1);

    // Optimization, no need to check each range's point
    for (edge1, edge2) in edges {
        if edge1.0 == edge2.0 && xran.contains(&edge1.0) {
            for y in edge1.1.min(edge2.1)..edge1.1.max(edge2.1) {
                if yran.contains(&y) {
                    return false;
                }
            }
        }

        if edge1.1 == edge2.1 && yran.contains(&edge1.1) {
            for x in edge1.0.min(edge2.0)..edge1.0.max(edge2.0) {
                if xran.contains(&x) {
                    return false;
                }
            }
        }
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

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

    let mut filtering_tiles = coords.clone();
    loop {
        let filtered_tiles = filtering_tiles
            .iter()
            .cloned()
            .filter(|coor| {
                for corner1 in &filtering_tiles {
                    for corner2 in &filtering_tiles {
                        if coor == corner1 || coor == corner2 {
                            continue;
                        }

                        let minx = corner1.0.min(corner2.0);
                        let maxx = corner1.0.max(corner2.0);
                        let miny = corner1.1.min(corner2.1);
                        let maxy = corner1.1.max(corner2.1);

                        if (minx..=maxx).contains(&coor.0) && (miny..=maxy).contains(&coor.1) {
                            return true;
                        }
                    }
                }

                false
            })
            .collect::<Vec<_>>();

        let filtered_anything = filtered_tiles.len() != filtering_tiles.len();
        filtering_tiles = filtered_tiles;
        dbg!(filtered_anything);
        if !filtered_anything {
            break;
        }
    }

    let mut area = 0;
    for coor1 in &coords {
        for coor2 in &coords {
            let dx = coor1.0.abs_diff(coor2.0) + 1;
            let dy = coor1.1.abs_diff(coor2.1) + 1;
            let this_area = dx * dy;

            let conjugate1 = (coor1.0, coor2.1);
            let conjugate2 = (coor2.0, coor1.1);

            if !is_really_contained_in_any_rectangle(conjugate1, &coords) { continue; }
            if !is_really_contained_in_any_rectangle(conjugate2, &coords) { continue; }

            // println!(
            //     "{},{} {},{} | {this_area}",
            //     coor1.0, coor1.1, coor2.0, coor2.1
            // );

            area = area.max(this_area);
        }
    }

    area
}

/// This should count
/// ....O.....O
/// .....X.....
/// ...........
/// ....O.....O
///
/// but not
/// ....O.....O
/// .....X.....
/// ...........
/// ..........O
///
/// I will refer to the second case as being normally included in a rectangle while the first case
/// is contained in both normally and inverse rectangles. Good way to identify is that dx and dy
/// have the same sign or not, if sgn(dx) == sgn(dy) its normal, otherwise antinormal.
fn is_really_contained_in_any_rectangle(coor: (usize, usize), inpoints: &[(usize, usize)]) -> bool {
    let mut found_normal = false;
    let mut found_inverse = false;

    for corner1 in inpoints {
        for corner2 in inpoints {
            let minx = corner1.0.min(corner2.0);
            let maxx = corner1.0.max(corner2.0);
            let miny = corner1.1.min(corner2.1);
            let maxy = corner1.1.max(corner2.1);

            if !((minx..=maxx).contains(&coor.0) && (miny..=maxy).contains(&coor.1)) {
                continue;
            }

            let dx = corner1.0 as isize - corner2.0 as isize;
            let dy = corner1.1 as isize - corner2.1 as isize;
            if dx == 0 || dy == 0 {
                return true;
            }
            if (dx * dy).is_positive() {
                found_normal = true;
            }
            if (dx * dy).is_negative() {
                found_inverse = true;
            }

            if found_inverse && found_normal {
                return true;
            }
        }
    }

    false
}

fn parse_ln(ln: &[u8]) -> (usize, usize) {
    let mut iter = ln.split(|&b| b == b',').map(|slice| {
        slice
            .iter()
            .fold(0, |acc, b| acc * 10 + (b - b'0') as usize)
    });

    (iter.next().unwrap(), iter.next().unwrap())
}

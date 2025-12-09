#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    // I do see how to make this in idk if O(n) or O(nlogn), but ima O(n^2) just at first
    let coords = buf[..(buf.len() - 1)]
        .split(|&b| b == b'\n')
        .map(parse_ln)
        .collect::<Vec<_>>();

    let mut area = 0;
    for coor1 in &coords {
        for coor2 in &coords {
            let dx = coor1.0.abs_diff(coor2.0) + 1;
            let dy = coor1.1.abs_diff(coor2.1) + 1;
            let this_area = dx * dy;

            // println!(
            //     "{},{} {},{} | {this_area}",
            //     coor1.0, coor1.1, coor2.0, coor2.1
            // );

            area = area.max(this_area);
        }
    }

    area
}

fn parse_ln(ln: &[u8]) -> (usize, usize) {
    let mut iter = ln.split(|&b| b == b',').map(|slice| {
        slice
            .iter()
            .fold(0, |acc, b| acc * 10 + (b - b'0') as usize)
    });

    (iter.next().unwrap(), iter.next().unwrap())
}

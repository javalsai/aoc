use std::mem;

fn is_paperroll(slice: &[u8], idx: usize) -> bool {
    slice.get(idx).is_some_and(|&x| x == b'@')
}

fn count_rolls_at_postns(
    slice: &[u8],
    idx: usize,
    postns: impl IntoIterator<Item = isize>,
) -> usize {
    let mut count = 0;
    for pos in postns.into_iter() {
        if is_paperroll(slice, idx.wrapping_add_signed(pos)) {
            count += 1;
        }
    }

    count
}

#[unsafe(no_mangle)]
fn challenge_usize(buf: &[u8]) -> usize {
    let mut prev_line = None;
    let mut surrounded_line: Option<&[u8]> = None;

    let mut total = 0;

    let mut cleaned_lines = Vec::new();

    for ln in buf[0..(buf.len() - 1)].split(|&b| b == b'\n') {
        if let Some(fln) = surrounded_line
            && prev_line.is_none()
        {
            let cleaned_start_idx = cleaned_lines.len();
            cleaned_lines.extend_from_slice(fln);
            cleaned_lines.push(b'\n');

            for (i, _) in fln.iter().enumerate().filter(|(_, b)| **b == b'@') {
                let adj_count = count_rolls_at_postns(fln, i, [-1, 1])
                    + count_rolls_at_postns(ln, i, [-1, 0, 1]);
                if adj_count < 4 {
                    cleaned_lines[cleaned_start_idx + i] = b'x';
                    total += 1;
                }
            }
        }

        if let Some(pln) = prev_line {
            let sln = surrounded_line.unwrap();

            let cleaned_start_idx = cleaned_lines.len();
            cleaned_lines.extend_from_slice(sln);
            cleaned_lines.push(b'\n');

            for (i, _) in sln.iter().enumerate().filter(|(_, b)| **b == b'@') {
                let adj_count = count_rolls_at_postns(pln, i, [-1, 0, 1])
                    + count_rolls_at_postns(sln, i, [-1, 1])
                    + count_rolls_at_postns(ln, i, [-1, 0, 1]);
                if adj_count < 4 {
                    cleaned_lines[cleaned_start_idx + i] = b'x';
                    total += 1;
                }
            }
        }

        prev_line = surrounded_line;
        surrounded_line = Some(ln);
    }

    let pln = prev_line.unwrap();
    let lln = surrounded_line.unwrap();

    let cleaned_start_idx = cleaned_lines.len();
    cleaned_lines.extend_from_slice(lln);
    cleaned_lines.push(b'\n');

    for (i, _) in lln.iter().enumerate().filter(|(_, b)| **b == b'@') {
        let adj_count =
            count_rolls_at_postns(pln, i, [-1, 0, 1]) + count_rolls_at_postns(lln, i, [-1, 1]);
        if adj_count < 4 {
            cleaned_lines[cleaned_start_idx + i] = b'x';
            total += 1;
        }
    }

    if total > 0 {
        // println!("{}", unsafe { str::from_utf8_unchecked(&cleaned_lines) });
        total += challenge_usize(&cleaned_lines);
    }

    total
}

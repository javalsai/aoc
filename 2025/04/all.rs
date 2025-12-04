// The use of these 2 could be optimized but not fatal performance

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
fn challenge_usize_duple(buf: &[u8]) -> (usize, usize) {
    let mut v = buf.to_vec();
    let trimmed_endl = v.len() - 1;

    let p1 = challenge_usize_inplace::<false>(&mut v[0..trimmed_endl]);

    let mut total = 0;
    let mut toadd = challenge_usize_inplace::<true>(&mut v[0..trimmed_endl]);
    while toadd > 0 {
        // println!("{}", unsafe { str::from_utf8_unchecked(&cleaned_lines) });
        total += toadd;
        toadd = challenge_usize_inplace::<true>(&mut v[0..trimmed_endl]);
    }

    (p1, total)
}

fn challenge_usize_inplace<const REPLACE: bool>(buf: &mut [u8]) -> usize {
    let mut prev_line: Option<&mut [u8]> = None;
    let mut surrounded_line: Option<&mut [u8]> = None;

    let mut total = 0;

    for ln in buf.split_mut(|&b| b == b'\n') {
        if let Some(fln) = surrounded_line {
            if prev_line.is_none() {
                for i in 0..fln.len() {
                    if fln[i] != b'@' {
                        continue;
                    }
                    let adj_count = count_rolls_at_postns(fln, i, [-1, 1])
                        + count_rolls_at_postns(ln, i, [-1, 0, 1]);
                    if adj_count < 4 {
                        if REPLACE {
                            fln[i] = b'x';
                        }
                        total += 1;
                    }
                }
            }
            surrounded_line = Some(fln);
        }

        if let Some(pln) = prev_line {
            let sln = surrounded_line.unwrap();

            for i in 0..sln.len() {
                if sln[i] != b'@' {
                    continue;
                }

                let adj_count = count_rolls_at_postns(pln, i, [-1, 0, 1])
                    + count_rolls_at_postns(sln, i, [-1, 1])
                    + count_rolls_at_postns(ln, i, [-1, 0, 1]);
                if adj_count < 4 {
                    if REPLACE {
                        sln[i] = b'x';
                    }
                    total += 1;
                }
            }

            surrounded_line = Some(sln);
        }

        prev_line = surrounded_line.take();
        surrounded_line = Some(ln);
    }

    let pln = prev_line.unwrap();
    let lln = surrounded_line.unwrap();

    for i in 0..lln.len() {
        if lln[i] != b'@' {
            continue;
        }

        let adj_count =
            count_rolls_at_postns(pln, i, [-1, 0, 1]) + count_rolls_at_postns(lln, i, [-1, 1]);
        if adj_count < 4 {
            if REPLACE {
                lln[i] = b'x';
            }
            total += 1;
        }
    }

    total
}

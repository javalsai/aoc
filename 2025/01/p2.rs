use std::{hint::unreachable_unchecked, io::BufRead};

#[unsafe(no_mangle)]
unsafe extern "Rust" fn challenge_isize(buf: &[u8]) -> isize {
    let mut count = 0;
    let mut pos = 50;

    for ln in buf.lines() {
        let ln = unsafe { ln.unwrap_unchecked() };
        let (dir, amt) = (ln.as_bytes()[0], &ln[1..]);

        let amt: isize = unsafe { amt.parse().unwrap_unchecked() };
        if amt == 0 {
            continue;
        }
        let prev_pos = pos;
        match dir {
            b'L' => pos -= amt,
            b'R' => pos += amt,
            _ => unsafe { unreachable_unchecked() },
        }

        count += if pos < 0 {
            if prev_pos == 0 {
                count -= 1;
            };
            (pos - 1).div_euclid(100).abs()
        } else if pos > 0 {
            pos.div_euclid(100)
        } else {
            1
        };
        pos = pos.rem_euclid(100);
    }

    count
}

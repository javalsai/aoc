use std::{hint::unreachable_unchecked, io::BufRead};

#[unsafe(no_mangle)]
extern "Rust" fn challenge_isize(buf: &[u8]) -> isize {
    let mut count = 0;
    let mut pos = 50;

    for ln in buf.lines() {
        let ln = unsafe { ln.unwrap_unchecked() };
        let (dir, amt) = unsafe { (ln.as_bytes().get_unchecked(0), ln.get_unchecked(1..)) };

        let amt: i16 = unsafe { amt.parse().unwrap_unchecked() };
        match dir {
            b'L' => pos -= amt,
            b'R' => pos += amt,
            _ => unsafe { unreachable_unchecked() },
        }

        pos %= 100;
        count += (pos != 0) as i16;
    }

    count.into()
}

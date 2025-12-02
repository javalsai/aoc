type SmolI = i32;

#[unsafe(no_mangle)]
unsafe extern "Rust" fn challenge_isize(buf: &[u8]) -> isize {
    let mut count = 0;
    let mut pos = 50;

    let buf = unsafe { str::from_utf8_unchecked(buf.get_unchecked(..(buf.len() - 1))) };

    for ln in buf.split('\n') {
        let (dir, amt) = unsafe { (*ln.as_bytes().get_unchecked(0), ln.get_unchecked(1..)) };

        let amt: SmolI = unsafe { amt.parse().unwrap_unchecked() };
        if amt == 0 { continue; }
        let prev_pos = pos;
        pos += ((((dir == b'R') as SmolI) << 1) - 1) * amt;

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

    count as isize
}

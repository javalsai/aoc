type SmolI = i16;

#[unsafe(no_mangle)]
extern "Rust" fn challenge_isize(buf: &[u8]) -> isize {
    let mut count = 0;
    let mut pos = 50;

    let buf = unsafe { str::from_utf8_unchecked(buf.get_unchecked(..(buf.len() - 1))) };

    for ln in buf.split('\n') {
        let (dir, amt) = unsafe { (*ln.as_bytes().get_unchecked(0), ln.get_unchecked(1..)) };

        let amt: SmolI = unsafe { amt.parse().unwrap_unchecked() };
        pos += ((((dir == b'R') as SmolI) << 1) - 1) * amt;
        pos %= 100;
        count += (pos == 0) as SmolI;
    }

    count as isize
}

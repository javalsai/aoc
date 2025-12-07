#[unsafe(no_mangle)]
pub extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    let mut splits = 0;

    let mut ln_iter = buf[0..(buf.len() - 1)].split(|&b| b == b'\n');

    let first_line = ln_iter.next().unwrap();
    let mut ray_boollist = Box::new_uninit_slice(first_line.len());
    first_line
        .iter()
        .map(|&c| c == b'S')
        .enumerate()
        .for_each(|(i, b)| {
            ray_boollist[i].write(b);
        });
    let mut ray_boollist = unsafe { ray_boollist.assume_init() };

    for ln in ln_iter {
        // dbg!(unsafe { str::from_utf8_unchecked(ln) }, splits);
        for (i, _) in ln.iter().enumerate().filter(|&(_, &c)| c == b'^') {
            if ray_boollist[i] {
                splits += 1;
                ray_boollist[i] = false;
                ray_boollist.get_mut(i - 1).map(|b| *b = true);
                ray_boollist.get_mut(i + 1).map(|b| *b = true);
            }
        }
    }

    splits
}

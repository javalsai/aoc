#[unsafe(no_mangle)]
fn challenge_usize(b: &[u8]) -> usize {
    let mut total = 0;
    let s = unsafe { str::from_utf8_unchecked(b) };

    dbg!(is_repeating(b"11"));
    let ranges = s.trim_end().split(',');
    for range in ranges {
        let (start, end) = range.split_once('-').unwrap();
        if start.starts_with('0') || end.starts_with('0') {
            continue;
        }
        let (start, end): (usize, usize) = (start.parse().unwrap(), end.parse().unwrap());
        println!("{start}-{end}");

        for id in start..=end {
            let id_s = id.to_string();

            if is_repeating(id_s.as_bytes()) {
                total += id;
                println!("-->> invalid {id_s}");
            }
        }
    }

    total
}

// fn is_repeating(s: &[u8]) -> bool {
//     if !s.len().is_multiple_of(2) {
//         return false;
//     }

//     let mid = s.len() / 2;
//     let (left, right) = s.split_at(mid);
//     left == right
// }

fn is_repeating(s: &[u8]) -> bool {
    for sublen in 0..(s.len() / 2) {
        let sublen = sublen + 1;
        if !s.len().is_multiple_of(sublen) {
            continue;
        }
        let pref = &s[0..sublen];

        let is_repeating = s[sublen..].chunks_exact(sublen).all(|c| c == pref);
        if is_repeating {
            return true;
        }
    }

    false
}

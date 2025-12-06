#[derive(Debug)]
enum Op {
    Mul,
    Add,
}

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    fn d(d: u8) -> usize {
        if d == b' ' { 0 } else { (d - b'0') as usize }
    }

    use Op::*;

    let mut iter = buf[0..(buf.len() - 1)]
        .rsplit(|&b| b == b'\n')
        .map(|ln| unsafe { str::from_utf8_unchecked(ln) }.split(' '));

    let mut ops = iter
        .next()
        .unwrap()
        .enumerate()
        .filter(|(_, s)| !s.is_empty())
        .map(|(i, op)| match op {
            "*" => (Mul, Vec::new(), i),
            "+" => (Add, Vec::new(), i),
            _ => unreachable!("invalid op"),
        })
        .collect::<Vec<_>>();

    iter.enumerate().for_each(|(bottom_idx, ln)| {
        let mut idx = 0;
        ln.filter_map(|n_str| {
            let r = (idx, n_str);
            idx += n_str.len() + 1;
            if n_str.is_empty() { None } else { Some(r) }
        })
        .zip(ops.iter_mut())
        .enumerate()
        .for_each(|(idx2, ((idx, n_str), acc))| {
            n_str.as_bytes().iter().enumerate().for_each(|(i, dig)| {
                let idx = idx + i - acc.2 - idx2;
                insert_dig(&mut acc.1, idx, *dig);
            });
        });
    });

    println!("{ops:?}");
    ops.iter().fold(0, |acc, (opacc, vec, _)| {
        let n: usize = match opacc {
            Mul => vec.iter().map(|(a, _)| a).product(),
            Add => vec.iter().map(|(a, _)| a).sum(),
        };
        acc + n
    })
}

fn insert_dig(into: &mut Vec<(usize, u32)>, at: usize, dig: u8) {
    if dig != b' ' {
        if let Some((mref, len)) = into.get_mut(at) {
            *mref += (dig - b'0') as usize * 10usize.pow(*len);
            dbg!(mref);
            *len += 1;
        } else {
            let mut i = into.len();
            while i < at {
                into.push((0, 0));
                i += 1;
            }
            into.push(((dig - b'0') as usize, 1));
        }
    }
}

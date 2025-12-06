#[derive(Clone, Copy, Debug)]
enum Op {
    Mul,
    Add,
}

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn challenge_usize(buf: &[u8]) -> usize {
    use Op::*;

    let mut iter = buf[0..(buf.len() - 1)].rsplit(|&b| b == b'\n').map(|ln| {
        unsafe { str::from_utf8_unchecked(ln) }
            .split(' ')
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
    });

    let mut ops = iter
        .next()
        .unwrap()
        .map(|op| match op {
            "*" => (Mul, 1),
            "+" => (Add, 0),
            _ => unreachable!("invalid op"),
        })
        .collect::<Vec<_>>();

    iter.for_each(|ln| {
        ln.zip(ops.iter_mut()).for_each(|(n, acc)| {
            let n: usize = n.parse().unwrap();

            match acc.0 {
                Mul => acc.1 *= n,
                Add => acc.1 += n,
            }
        });
    });

    ops.iter().fold(0, |acc, (_, n)| acc + n)
}

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(static_mut_refs)]

use core::{
    arch::{asm, naked_asm},
    cmp,
    ffi::CStr,
    fmt::{self, Debug, Write},
    mem, ops,
    panic::PanicInfo,
    usize,
};
// use std::{fs::File, io::Read};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CInstant {
    secs: i64,  // time_t
    nanos: i64, // long
}

impl CInstant {
    fn now() -> Self {
        let clockid: isize = 1; // CLOCK_MONOTONIC (non-jumping-backwards uptime)
        let instant = unsafe {
            let mut instant: Self = mem::zeroed();
            let mut result: isize;

            asm!(
                "syscall",
                in("rax") 228,
                in("rsi") &mut instant,
                in("rdi") clockid,
                lateout("rax") result,
                out("rcx") _,
                out("r11") _,
                options(nostack)
            );

            if result < 0 {
                Err(result)
            } else {
                Ok(instant)
            }
        };

        instant.expect("cant even trust syscalls on stack nowadays")
    }

    pub fn as_nanos(self) -> i128 {
        (self.secs as i128) * 1_000_000_000 + (self.nanos as i128)
    }
}

impl ops::Sub for CInstant {
    type Output = CDuration;
    fn sub(self, rhs: Self) -> Self::Output {
        CDuration::from(self.as_nanos() - rhs.as_nanos())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CDuration {
    pub nanos: i128,
}

const IS_SUB_UNITS: &[char] = &['n', 'μ', 'm'];
impl Debug for CDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        let max_idx = 3;
        let mut amount = self.nanos;

        while amount >= 1000 && i < max_idx {
            amount /= 1000;
            i += 1;
        }

        write!(f, "{amount}")?;
        if i != 3 {
            write!(f, "{}", IS_SUB_UNITS[i])?;
        }

        write!(f, "s")
    }
}

impl CDuration {
    pub const ZERO: CDuration = CDuration::from(0);
    pub const MAX: CDuration = CDuration::from(i128::MAX);

    pub const fn from(nanos: i128) -> Self {
        Self { nanos }
    }
}

impl ops::DivAssign<u32> for CDuration {
    fn div_assign(&mut self, rhs: u32) {
        self.nanos /= rhs as i128
    }
}

impl ops::Div<u32> for CDuration {
    type Output = CDuration;
    fn div(mut self, rhs: u32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl ops::AddAssign for CDuration {
    fn add_assign(&mut self, rhs: Self) {
        self.nanos += rhs.nanos;
    }
}

impl ops::Neg for CDuration {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        self.nanos = -self.nanos;
        self
    }
}

// FD
struct FDWriter(usize);

impl FDWriter {
    pub const fn from_fd(fd: usize) -> Self {
        Self(fd)
    }
}

impl Write for FDWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let result: isize;
            asm!(
                "syscall",
                in("rax") 1, // write
                in("rdi") self.0, // fd
                in("rsi") s.as_ptr() as *const u8, // *data
                in("rdx") s.as_bytes().len(), // data len
                lateout("rax") result,
                out("rcx") _,
                out("r11") _,
                options(nostack)
            );

            if result < 0 {
                Err(fmt::Error)
            } else {
                Ok(())
            }
        }
    }
}

#[naked]
#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        naked_asm!(
            "mov {0}, rsp",
            "jmp {1}",
            sym STACK_START,
            sym main
        );
    }
}

static mut ARGC: usize = 1;
static mut STACK_START: *const () = unsafe { mem::zeroed() };
fn get_arg(idx: usize) -> Option<&'static CStr> {
    unsafe {
        if idx >= ARGC {
            return None;
        }

        let ptr = ((STACK_START as usize) + (idx * 8) + 8) as *const *const i8;
        let cstr = core::ffi::CStr::from_ptr(*ptr);
        Some(cstr)
    }
}

pub fn exit(code: isize) -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") code,
            options(nostack, noreturn)
        );
    }
}

static mut STDOUT: FDWriter = FDWriter::from_fd(1);
static mut STDERR: FDWriter = FDWriter::from_fd(2);
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        _ = writeln!(&mut STDERR, "\x1b[1;31m{info:?}\x1b[0m");
    }
    exit(1);
}

pub fn unsafe_sized_slice_collect<T, I: Iterator<Item = T>>(slice: &mut [T], iter: I) -> usize {
    let mut i = 0;
    for e in iter {
        slice[i] = e;
        i += 1;
    }
    i
}

pub extern "C" fn main() -> ! {
    unsafe {
        ARGC = *(STACK_START as *const usize);
    }

    // let args: Vec<_> = std::env::args().collect();
    // let filename = &args[1];
    let runs: u32 = get_arg(1)
        .expect("no arg 1")
        .to_str()
        .expect("invalid string encoding")
        .parse()
        .expect("did not receive a number as `runs` argument");

    // let mut contents = vec![];
    // let mut file = File::open(filename)?;
    // file.read_to_end(&mut contents)?;
    let contents: &[u8] = include_bytes!("../input.txt");

    let mut lines: [&[u8]; 1024] = [&[]; 1024];
    let len = unsafe_sized_slice_collect(&mut lines, contents.split(|&b| b == b'\n'));

    let mut max = CDuration::ZERO;
    let mut avg = CDuration::ZERO;
    let mut min = CDuration::MAX;
    _ = writeln!(
        unsafe { &mut STDOUT },
        "\x1b[0;1;31;43m[W]\x1b[0;1;33m syscall time {:?}\x1b[0m",
        -(CInstant::now() - CInstant::now())
    );
    let result = run(&lines, len);
    for i in 0..runs {
        let a = CInstant::now();
        let run_result = run(&lines, len);
        let b = CInstant::now();
        if result != run_result {
            panic!(
                "supposed result is {} but this run got {} after {} runs",
                result, run_result, i
            );
        }

        let d = b - a;
        max = cmp::max(d, max);
        min = cmp::min(d, min);
        avg += d / runs;
    }

    _ = writeln!(unsafe { &mut STDOUT }, "\x1b[0;34mtotal \x1b[1;33m{result}\x1b[0;34m\n  runs  \x1b[1;33m{runs:?}\x1b[0;34m\n  avg.  \x1b[1;35m{avg:?}\x1b[0;34m\n  max.  \x1b[1;31m{max:?}\x1b[0;34m\n  min.  \x1b[1;32m{min:?}\x1b[0m");

    exit(0);
}

fn run(lines: &[&[u8]], len: usize) -> usize {
    let mut t = 0;
    for i in 0..len {
        let ln = lines[i];
        for (j, &ch) in ln.iter().enumerate() {
            if ch == b'X' {
                for dir in [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ] {
                    let r = eq_in_dir(lines, (j, i), dir, "MAS".bytes());
                    if r {
                        t += 1;
                    }
                }
            }
        }
    }
    t
}

macro_rules! pstve_try_bool {
    ($a:expr, $b:expr) => {{
        let r = $a + $b;
        if r < 0 {
            return false;
        }
        r
    }};
}

fn eq_in_dir(
    buf: &[&[u8]],
    pos: (usize, usize),
    dir: (isize, isize),
    mut matches: impl Iterator<Item = u8>,
) -> bool {
    let Some(must_match) = matches.next() else {
        return true;
    };

    let new_1 = pstve_try_bool!(pos.1 as isize, dir.1) as usize;
    let new_0 = pstve_try_bool!(pos.0 as isize, dir.0) as usize;
    let Some(ln) = buf.get(new_1) else {
        return false;
    };
    let Some(ch) = ln.get(new_0) else {
        return false;
    };

    if must_match == *ch {
        eq_in_dir(buf, (new_0, new_1), dir, matches)
    } else {
        false
    }
}

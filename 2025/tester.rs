#![feature(iter_next_chunk, ptr_metadata, fn_ptr_trait)]

use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{self, Duration, Instant},
};

pub mod dl {
    use std::{
        ffi::{CStr, CString},
        fmt::Pointer,
        marker::FnPtr,
        os::unix::ffi::OsStrExt,
        path::Path,
        ptr::{NonNull, Pointee},
        str::Utf8Error,
    };

    const RTLD_NOW: i32 = 1;
    const RTLD_GLOBAL: i32 = 2;

    #[derive(Debug)]
    pub struct Handle(NonNull<ffi::Handle>);

    impl Handle {
        /// To get the error call [`ffi::error()`].
        pub fn try_drop(self) -> Result<(), (Self, i32)> {
            let err = unsafe { ffi::close(self.0) };
            if err == 0 { Ok(()) } else { Err((self, err)) }
        }

        /// # Safety
        ///
        /// There's no guarantees the pointer is actually of `T` aside from the generic.
        pub unsafe fn sym<T: Pointee>(&self, symbol: &CStr) -> Option<NonNull<T>> {
            NonNull::new(unsafe { ffi::sym(self.0, symbol.as_ptr().cast()).cast() })
        }

        /// # Safety
        ///
        /// There's no guarantees the pointer is actually of `T` aside from the generic.
        pub unsafe fn symfn<T: FnPtr + Pointer>(&self, symbol: &CStr) -> Option<T> {
            let ptr = unsafe { ffi::sym(self.0, symbol.as_ptr().cast()) };
            if ptr.is_null() {
                None
            } else {
                Some(unsafe { std::mem::transmute_copy(&ptr) })
            }
        }
    }

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { ffi::close(self.0) };
        }
    }

    #[derive(Debug)]
    pub struct DlOpenError;

    /// To get the error call [`ffi::error()`].
    pub fn open(path: impl AsRef<Path>) -> Result<Handle, DlOpenError> {
        let osstr = path.as_ref().as_os_str();
        let cstr =
            CString::new(osstr.as_bytes()).expect("Failed to make CString from path's OSString");

        let raw_handle = unsafe { ffi::open(cstr.as_ptr() as *const _, RTLD_NOW | RTLD_GLOBAL) };
        if let Some(raw_handle) = NonNull::new(raw_handle) {
            Ok(Handle(raw_handle))
        } else {
            Err(DlOpenError)
        }
    }

    /// # Safety
    ///
    /// There must be a previous error, will assume [`ffi::error()`] is not null. And the returned
    /// slice will only be valid until another call to dl happens.
    pub unsafe fn last_error() -> Result<&'static str, Utf8Error> {
        let cstr = unsafe { CStr::from_ptr(ffi::error()) };
        cstr.to_str()
    }

    pub mod ffi {
        use std::{
            ffi::{c_char, c_void},
            ptr::NonNull,
        };

        #[repr(C)]
        pub struct Handle(());

        #[link(name = "dl")]
        unsafe extern "C" {
            #[link_name = "dlopen"]
            pub fn open(path: *const c_char, flags: i32) -> *mut Handle;
            #[link_name = "dlclose"]
            pub fn close(handle: NonNull<Handle>) -> i32;
            #[link_name = "dlsym"]
            pub fn sym(handle: NonNull<Handle>, symbol: *const c_char) -> *mut c_void;
            /// NULL if there was no error, may also return an address to a static buffer so its a
            /// MUST to not call it again until this buffer is cloned or freed and no referenced to
            /// it remain.
            #[link_name = "dlerror"]
            pub fn error() -> *mut c_char;
        }
    }
}

pub mod loader {
    use std::time::{Duration, Instant};

    use crate::dl::Handle;

    pub type ChallengeFn<T> = unsafe extern "Rust" fn(&[u8]) -> T;
    pub type TimedChallengeFn<T> = unsafe extern "Rust" fn(&[u8], &Instant) -> T;

    pub enum FnVariant {
        Isize(ChallengeFn<isize>),
        Usize(ChallengeFn<usize>),
        IsizeDuple(ChallengeFn<(isize, isize)>),
        UsizeDuple(ChallengeFn<(usize, usize)>),
        TIsize(TimedChallengeFn<isize>),
        TUsize(TimedChallengeFn<usize>),
        TIsizeDuple(TimedChallengeFn<(isize, isize)>),
        TUsizeDuple(TimedChallengeFn<(usize, usize)>),
    }

    #[derive(Debug, PartialEq)]
    pub enum FnRetVariant {
        Isize(isize),
        Usize(usize),
        IsizeDuple((isize, isize)),
        UsizeDuple((usize, usize)),
    }

    impl FnVariant {
        /// Ideally the fn would be put on the heap, but that requires customly allocating
        /// executable memory, a DST pain other stuff I can't be concerned with rn.
        pub const fn make_noop_stub_isize() -> Self {
            unsafe extern "Rust" fn __stub(_: &[u8]) -> isize {
                0
            }

            FnVariant::Isize(__stub)
        }

        /// # Safety
        ///
        /// Calls external arbitrary FNs.
        pub unsafe fn call(&self, buf: &[u8], instant: &Instant) -> FnRetVariant {
            use FnRetVariant as R;
            use FnVariant as V;

            match self {
                V::Isize(f) => unsafe { R::Isize((f)(buf)) },
                V::Usize(f) => unsafe { R::Usize((f)(buf)) },
                V::IsizeDuple(f) => unsafe { R::IsizeDuple((f)(buf)) },
                V::UsizeDuple(f) => unsafe { R::UsizeDuple((f)(buf)) },
                V::TIsize(f) => unsafe { R::Isize((f)(buf, instant)) },
                V::TUsize(f) => unsafe { R::Usize((f)(buf, instant)) },
                V::TIsizeDuple(f) => unsafe { R::IsizeDuple((f)(buf, instant)) },
                V::TUsizeDuple(f) => unsafe { R::UsizeDuple((f)(buf, instant)) },
            }
        }

        /// # Safety
        ///
        /// Calls external arbitrary FNs.
        pub unsafe fn call_notimer(&self, buf: &[u8]) -> Option<FnRetVariant> {
            use FnRetVariant as R;
            use FnVariant as V;

            match self {
                V::Isize(f) => Some(unsafe { R::Isize((f)(buf)) }),
                V::Usize(f) => Some(unsafe { R::Usize((f)(buf)) }),
                V::IsizeDuple(f) => Some(unsafe { R::IsizeDuple((f)(buf)) }),
                V::UsizeDuple(f) => Some(unsafe { R::UsizeDuple((f)(buf)) }),
                _ => None,
            }
        }
    }

    pub fn load_fn_from(handle: &Handle) -> Option<FnVariant> {
        macro_rules! untry {
            ($e:expr) => {
                match $e {
                    Some(v) => return Some(v),
                    None => (),
                }
            };
        }

        use ChallengeFn as C;
        use FnVariant as V;
        use TimedChallengeFn as TC;

        // SAFETY: This is actually safe as it just creates the fn ptr, no weird fn with weird
        // drop.
        untry!(unsafe { handle.symfn::<C<isize>>(c"challenge_isize") }.map(V::Isize));
        untry!(unsafe { handle.symfn::<C<usize>>(c"challenge_usize") }.map(V::Usize));
        untry!(
            unsafe { handle.symfn::<C<(isize, isize)>>(c"challenge_isize_duple") }
                .map(V::IsizeDuple)
        );
        untry!(
            unsafe { handle.symfn::<C<(usize, usize)>>(c"challenge_usize_duple") }
                .map(V::UsizeDuple)
        );

        untry!(unsafe { handle.symfn::<TC<isize>>(c"challenge_t_isize") }.map(V::TIsize));
        untry!(unsafe { handle.symfn::<TC<usize>>(c"challenge_t_usize") }.map(V::TUsize));
        untry!(
            unsafe { handle.symfn::<TC<(isize, isize)>>(c"challenge_t_isize_duple") }
                .map(V::TIsizeDuple)
        );
        untry!(
            unsafe { handle.symfn::<TC<(usize, usize)>>(c"challenge_t_usize_duple") }
                .map(V::TUsizeDuple)
        );

        None
    }

    pub fn load_timers_from(handle: &Handle) -> Option<&[(&str, Duration)]> {
        // SAFETY: This is actually safe as it just creates the fn ptr, no weird fn with weird
        // drop.
        let at = unsafe { handle.sym::<(&str, Duration)>(c"TIMERS") }?;
        let len = unsafe { handle.sym::<usize>(c"TIMERS_LEN") }?;
        let len = unsafe { *len.as_ref() };

        Some(unsafe { std::slice::from_raw_parts(at.as_ptr(), len) })
    }
}

pub mod performer {
    use std::time::{Duration, Instant};

    pub fn get_avg_runt(runs: u32, run: fn()) -> std::time::Duration {
        let mut total_t = Duration::ZERO;
        for _ in 0..runs {
            let t = Instant::now();
            run();
            total_t += t.elapsed();
        }

        total_t / runs
    }
}

macro_rules! info {
    ($fmt:literal) => {
        println!(concat!("\x1b[34m inf: ", $fmt, "\x1b[0m"));
    };

    ($fmt:literal, $($arg:tt)*) => {
        println!(concat!("\x1b[34m inf: ", $fmt, "\x1b[0m"), $($arg)*);
    };
}

pub type ChallengeFn = unsafe extern "Rust" fn(&[u8]) -> isize;

fn main() -> io::Result<()> {
    let rustc = env::var("RUSTC_PATH").ok();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("\x1b[1;31mUsage: $0 <PATHS.RS> <FILEPATH|->");
        return Ok(());
    };

    let [days @ .., input] = &args[..] else {
        unreachable!("already checked arg count before");
    };

    let (noop_overhead_cold, noop_overhead_hot) = measure_noop_overhead();
    info!(
        "noop fn takes {:#?} hot and {:#?} cold",
        noop_overhead_hot, noop_overhead_cold
    );
    println!("\x1b[33m prog: buferring input...\x1b[0m");
    let input = buffer_input(input)?;

    if days.len() == 1 {
        println!();
        run_input(&days[0], &input, rustc.as_deref())
    } else {
        for day in days {
            println!();
            println!("\x1b[1;3;4;41m{day}\x1b[0m:");
            run_input(day, &input, rustc.as_deref())?;
        }

        Ok(())
    }
}

fn run_input(day: &str, input: &[u8], rustc: Option<&str>) -> io::Result<()> {
    let rs_path = PathBuf::from(&day);
    assert_eq!(rs_path.extension(), Some(OsStr::new("rs")));
    let so_path = rs_path.with_extension("so");

    compile(&rs_path, &so_path, rustc)?;

    let challenge = dl::open(so_path).expect("Couldn't load dyn library");
    let challenge_main = loader::load_fn_from(&challenge).expect(
        "Didn't find any appropiate symbol in the compiled .so file. Make sure there is one.",
    );

    let start = Instant::now();
    let result = unsafe { challenge_main.call(input, &start) };

    let total_time = start.elapsed();
    println!(
        "\x1b[32mdone\x1b[0m in \x1b[35m{total_time:#?}\x1b[0m and yielded result \x1b[36m{result:?}\x1b[0m",
    );
    if let Some(timers) = loader::load_timers_from(&challenge) {
        let mut prev_t = Duration::ZERO;
        for (name, cum_timer) in timers {
            let timer = *cum_timer - prev_t;
            println!(
                " '\x1b[36m{name}\x1b[0m': \x1b[35m{timer:#?}\x1b[0m (\x1b[1;33m{:.2}%\x1b[0m) \x1b[35m{cum_timer:#?}\x1b[0m (\x1b[1;33m{:.2}%\x1b[0m)",
                (timer.as_nanos() as f64 / total_time.as_nanos() as f64) * 100.0,
                (cum_timer.as_nanos() as f64 / total_time.as_nanos() as f64) * 100.0
            );

            prev_t = *cum_timer;
        }
    }

    Ok(())
}

const STUB_ISIZE: loader::FnVariant = loader::FnVariant::make_noop_stub_isize();
fn measure_noop_overhead() -> (time::Duration, time::Duration) {
    let timer = Instant::now();
    // SAFETY: completely safe, not arbitrary fn but our stub
    unsafe { STUB_ISIZE.call(&[], &timer) };
    let cold = timer.elapsed();
    let hot = performer::get_avg_runt(u16::MAX.into(), || unsafe {
        STUB_ISIZE.call_notimer(&[]);
    });

    (cold, hot)
}

fn compile(rs: &Path, so: &Path, rustc: Option<&str>) -> io::Result<()> {
    if env::var("REBUILD").is_err() {
        let rsf = File::open(rs)?;
        let rs_metadata = rsf.metadata()?;

        if let Ok(f) = File::open(so) {
            let so_metadata = f.metadata()?;
            so_metadata.modified()?;

            // No need to recompile
            if so_metadata.modified()? > rs_metadata.modified()? {
                println!("\x1b[32m ok: challenge already compiled\x1b[0m");
                return Ok(());
            }
        }
    }

    // Recompile
    println!("\x1b[33m prog: compiling {rs:#?}...\x1b[0m");
    let exit = Command::new(rustc.unwrap_or("rustc"))
        .args([
            "--crate-type=cdylib",
            "--edition=2024",
            "-O",
            "-Copt-level=3",
            "-Cstrip=symbols",
            "-Cdebuginfo=0",
            "-Coverflow-checks=false",
            "-Cpanic=abort",
            "-Ctarget-cpu=native",
            "-Ccodegen-units=1",
            "-Cdebug-assertions=off",
        ])
        .arg(rs)
        .arg("-o")
        .arg(so)
        .stdout(Stdio::inherit())
        .spawn()?
        .wait()?;

    if exit.success() {
        Ok(())
    } else {
        Err(io::Error::other("rustc command failed"))
    }
}

fn buffer_input(input_path: &str) -> io::Result<Vec<u8>> {
    let mut input = Vec::new();

    if input_path == "-" {
        io::stdin().read_to_end(&mut input)?;
        Ok(input)
    } else {
        let t = Instant::now();
        let mut f = File::open(input_path)?;
        f.read_to_end(&mut input)?;
        info!("\x1b[34mbuffering input {:#?}\x1b[0m", t.elapsed());

        Ok(input)
    }
}

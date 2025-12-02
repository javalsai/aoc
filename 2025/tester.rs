#![feature(iter_next_chunk, ptr_metadata, fn_ptr_trait)]

use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{self, Instant},
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
            if err == 0 {
                Ok(())
            } else {
                Err((self, err))
            }
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
    use crate::dl::Handle;

    pub type ChallengeFn<T> = unsafe extern "Rust" fn(&[u8]) -> T;

    pub enum FnVariant {
        Isize(ChallengeFn<isize>),
        Usize(ChallengeFn<usize>),
        IsizeDuple(ChallengeFn<(isize, isize)>),
        UsizeDuple(ChallengeFn<(usize, usize)>),
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
        pub unsafe fn call(&self, buf: &[u8]) -> FnRetVariant {
            use FnRetVariant as R;
            use FnVariant as V;

            match self {
                V::Isize(f) => unsafe { R::Isize((f)(buf)) },
                V::Usize(f) => unsafe { R::Usize((f)(buf)) },
                V::IsizeDuple(f) => unsafe { R::IsizeDuple((f)(buf)) },
                V::UsizeDuple(f) => unsafe { R::UsizeDuple((f)(buf)) },
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

        // SAFETY: This is actually safe as it just creates the fn ptr, no weird fn with weird
        // drop.
        untry!(unsafe { handle.symfn::<C<isize>>(c"challenge_isize") }.map(V::Isize));
        untry!(unsafe { handle.symfn::<C<usize>>(c"challenge_isize") }.map(V::Usize));
        untry!(unsafe { handle.symfn::<C<(usize, usize)>>(c"challenge_isize_duple") }.map(V::UsizeDuple));
        untry!(unsafe { handle.symfn::<C<(isize, isize)>>(c"challenge_isize_duple") }.map(V::IsizeDuple));

        None
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

pub type ChallengeFn = unsafe extern "Rust" fn(&[u8]) -> isize;

fn main() -> io::Result<()> {
    let rustc = env::var("RUSTC_PATH").ok();

    let Ok([day, input]) = std::env::args().skip(1).next_chunk() else {
        eprintln!("\x1b[1;31mUsage: $0 <PATH.RS> <FILEPATH|->");
        return Ok(());
    };

    let (noop_overhead_cold, noop_overhead_hot) = measure_noop_overhead();
    println!(
        "\x1b[34minf: noop fn takes {noop_overhead_hot:#?} hot and {noop_overhead_cold:#?} cold\x1b[0m",
    );

    let rs_path = PathBuf::from(&day);
    assert_eq!(rs_path.extension(), Some(OsStr::new("rs")));
    let so_path = rs_path.with_extension("so");

    compile(&rs_path, &so_path, rustc.as_deref())?;
    println!("\x1b[33mBuferring input...\x1b[0m");
    let input = buffer_input(&input)?;

    let challenge = dl::open(so_path).expect("Couldn't load dyn library");
    let challenge_main = loader::load_fn_from(&challenge).expect(concat!(
        "Didn't find any appropiate symbol in the compiled .so file. Make sure there is one and is ",
        stringify!(unsafe extern "Rust" fn(&[u8]) -> isize)
    ));

    let start = Instant::now();
    let result = unsafe { challenge_main.call(&input) };

    println!(
        "done in {:#?} and yielded result {:?}",
        start.elapsed(),
        result
    );

    Ok(())
}

const STUB_ISIZE: loader::FnVariant = loader::FnVariant::make_noop_stub_isize();
fn measure_noop_overhead() -> (time::Duration, time::Duration) {
    let timer = Instant::now();
    // SAFETY: completely safe, not arbitrary fn but our stub
    unsafe { STUB_ISIZE.call(&[]) };
    let cold = timer.elapsed();
    let hot = performer::get_avg_runt(u16::MAX.into(), || unsafe {
        STUB_ISIZE.call(&[]);
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
                println!("\x1b[32mChallenge already compiled\x1b[0m");
                return Ok(());
            }
        }
    }

    // Recompile
    println!("\x1b[33mCompiling {rs:#?}...\x1b[0m");
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
        let mut f = File::open(input_path)?;
        f.read_to_end(&mut input)?;
        Ok(input)
    }
}

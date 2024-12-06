# AOC
What is AOC??: [adventofcode](https://adventofcode.com)

All this is a mix of langs, but in general:
* `asm`: I use nasm, at least for now for assembly programs (linux, x86\_64), no special linking normallly, so just elf64 compile and link.
* `zig`: No need to set up a full project, simply `zig build-exe -O ReleaseFast -fsingle-threaded -fno-unwind-tables -fno-error-tracing -fno-formatted-panics -fstrip -fno-stack-protector main.zig` for max performance.
* `rust`: No need for projects neither, also `rustc -Copt-level=3 -Ctarget-cpu=native -Cpanic=abort -Cstrip=symbols -Coverflow_checks=n -Clto=fat` for squeezing performance.

I also tend to use `perf stat` to check their performance.

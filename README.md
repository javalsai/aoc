# AOC
What is AOC??: [adventofcode](https://adventofcode.com)

All this is a mix of langs, but in general:
* `asm`: I use nasm, at least for now for assembly programs (linux, x86\_64), no special linking normallly, so just elf64 compile and link.
* `zig`: No need to set up a full project, simply `zig build-exe -fstrip -OReleaseFast main.zig`

I also tend to use `perf stat` to check their performance.

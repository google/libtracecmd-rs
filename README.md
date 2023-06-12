# libtracecmd-rs

**A Rust wrapper of [libtracecmd](https://github.com/rostedt/trace-cmd/tree/master/lib/trace-cmd).**

[<img alt="crates.io" src="https://img.shields.io/crates/v/libtracecmd.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/libtracecmd)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/google/libtracecmd-rs/build.yml?branch=main&style=for-the-badge" height="20">](https://github.com/google/libtracecmd-rs/actions?query=branch%3Amain)

This library is a Rust wrapper of [libtracecmd](https://www.trace-cmd.org/Documentation/libtracecmd/),
which allows writing programs to analyze Linux's [ftrace](https://docs.kernel.org/trace/ftrace.html)
data recoreded by [trace-cmd](https://github.com/rostedt/trace-cmd).

## Requirements

To use this crate in your program, you need to install the [libtracecmd](https://github.com/rostedt/trace-cmd) library (>= 1.2.0) on your system.

## Example Usage

Let's see how it works with [examples/top_n_events](https://github.com/google/libtracecmd-rs/blob/main/examples/top_n_events.rs),
which counts how many times each event occurred in a particular period.

First, create `trace.dat` by running `trace-cmd`.

```sh
trace-cmd record -e syscalls sleep 10
```

Then, show the top-10 syscall events in the trace.dat file.

```sh
cargo run --example top_n_events -- --input trace.dat --n 10 --prefix sys_enter_
```

Example output:

```
Top 10 events:
#1: ioctl: 62424 times
#2: futex: 59074 times
#3: read: 30144 times
#4: write: 28361 times
#5: newfstatat: 22590 times
#6: close: 15893 times
#7: splice: 14650 times
#8: getuid: 13579 times
#9: epoll_pwait: 12298 times
#10: ppoll: 10523 times
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.

## License

Apache 2.0; see [`LICENSE`](LICENSE) for details.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.

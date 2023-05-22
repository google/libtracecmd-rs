# libtracecmd Rust wrapper

A safe Rust wrapper of [libtracecmd](https://github.com/rostedt/trace-cmd/tree/master/lib/trace-cmd).

## How to run

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

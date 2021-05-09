# Play musical notes from any computer keyboard

I built this crate so that my baby could play multiple
notes at the same time just by using a computer keyboard
I already had. Basic music toys we had only played one 
note at a time so it was an excuse for a side project.

# Setup

For those new to Rust: download the toolchain [here](https://rustup.rs)

Run the program: `cargo run --release` (debug profile can cause lag in
generating stream)

# Mapping keys to frequencies

Given the scan code of the key pressed, the `row` and 
`column` number are calculated for scan codes less than
64 (mostly the alphabetic keys and numeric keys above).
This crate configures the frequencies of neighbouring
keys to have strong consonance: neighbour to the right
has frequency 4/3 times higher (perfect fourth) and
upper right diagonal neighbour has frequency 3/2 higher
(perfect fifth). 

# Use of external crates

- The `cpal` crate is used to play audio streams which
this crate generates based on what keys are pressed.
- The `winit` crate is used to handle key press and 
release events.
- An unbounded multi-producer multi-consumer channel from the 
`async-std` crate is used to pass messages between 
`winit`'s `EventLoop` and `dashmap`'s `DashSet` to store
the set of keys pressed.
- The `Lazy` wrapper from `once_cell` is used to
arbitrarily construct static variables to satisfy Rust's
borrow checker


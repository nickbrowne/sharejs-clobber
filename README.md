# sharejs-clobber

Multi-threaded sharejs benchmark for text document inserts.

## Usage

Clone the repo and `cd` into it.

```
cargo run --release
```

Assumes you have sharejs running locally on port 9000. If not, go hack the urls in `doc.rs`.

Change the `CONCURRENCY` constant as you see fit.

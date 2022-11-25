An unfinished Rust port of https://github.com/oisyn/parkerwords/ C++ implementation.

Please note that it is not fair to compare the reading time, since the text file is basically built into the binary here.
Everything else should be comparable.

This implementation

```
$ cargo build --release

$ ./target/release/parkerwords-rs
[src\main.rs:183] ctx.all_word_bits.len() = 5977
538 solutions written to solutions.txt
Total time: 19.2388ms
Read:       6.8434ms
Process:    11.8048ms
Write:      590.6µs

$ ./target/release/parkerwords-rs
[src\main.rs:183] ctx.all_word_bits.len() = 5977
538 solutions written to solutions.txt
Total time: 18.8244ms
Read:       6.7889ms
Process:    11.3585ms
Write:      677µs

$ ./target/release/parkerwords-rs.exe
[src\main.rs:183] ctx.all_word_bits.len() = 5977
538 solutions written to solutions.txt
Total time: 18.9783ms
Read:       6.8236ms
Process:    11.6318ms
Write:      522.9µs
```

Original C++ implementation

```
$ g++ parkerwords.cpp -std=c++20 -O3 -o parkerwords

$ ./parkerwords
538 solutions written to solutions.txt.
Total time: 29999us (0.029999s)
Read:       11998us
Process:    17001us
Write:       1000us

$ ./parkerwords
538 solutions written to solutions.txt.
Total time: 25999us (0.025999s)
Read:       10998us
Process:    14000us
Write:       1001us

$ ./parkerwords
538 solutions written to solutions.txt.
Total time: 27000us (0.027s)
Read:       10998us
Process:    15004us
Write:        998us

$ ./parkerwords
538 solutions written to solutions.txt.
Total time: 28020us (0.02802s)
Read:       10984us
Process:    17036us
Write:          0us
```

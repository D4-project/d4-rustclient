# d4-rustclient
Very basic and non functional rust client (WiP)

# Installation

`cargo build`

# Help

```
cargo run -- -h
```

# Usage

```
$  echo "blahh" | cargo run -- -c <config_directory> | xxd
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/d4-rustclient`
00000000: 0101 388b 27e2 92b5 41ea 8469 a609 097a  ..8.'...A..i...z
00000010: ea59 1cd6 ee60 0000 0000 f671 2581 67de  .Y...`.....q%.g.
00000020: 7441 8124 248e 5bed 9607 f2d5 ad54 d617  tA.$$.[......T..
00000030: c1df be41 9d5e d04d 31f9 0600 0000 626c  ...A.^.M1.....bl
00000040: 6168 680a                                ahh.
```

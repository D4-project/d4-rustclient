# d4-rustclient
Very basic rust client.

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

# Python bindings

```
poetry install
poetry run nosetests-3.4 tests/tests_python.py
```

Have a look at `tests/tests_python.py` for a quick example.

# Python script

```
$ (git::main) echo -n blah | d4client-rust | xxd
00000000: 0101 cd98 0d75 dcf1 47da bb60 aabe 94fe  .....u..G..`....
00000010: a6b2 d37a 1f61 0000 0000 e6d1 f434 d8da  ...z.a.......4..
00000020: a2e0 5ca1 4dca 5407 ffd4 c45d 0522 273a  ..\.M.T....]."':
00000030: fc8c b728 a24b 1b61 f06a 0400 0000 626c  ...(.K.a.j....bl
00000040: 6168                                     ah
```

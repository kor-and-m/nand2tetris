# Two tier compiler from https://www.nand2tetris.org/

## Translate .vm to hack .asm
```
cd rust_code
cargo run -- ../static/vm/FibonacciElement
cat ../static/vm/FibonacciElement/FibonacciElement.asm
```

## Translate .vm to .hack (preaty printed)
```
cd rust_code
TO_BINARY=1 cargo run -- ../static/vm/FibonacciElemen
cat ../static/vm/FibonacciElement/FibonacciElement.hack
```

## TODO
1. Translate .jack to .vm => Translate .jack to .hack
2. Translate .jack to llvm format

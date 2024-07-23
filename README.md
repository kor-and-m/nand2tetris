# Two tier compiler for https://www.nand2tetris.org/ project

## Translate .vm to hack .asm
```
cd rust_code
cargo run -p vm_translator -- ../static/vm/FibonacciElement
cat ../static/vm/FibonacciElement/FibonacciElement.asm
```

## Translate .vm to .hack (preaty printed)
```
cd rust_code
TO_BINARY=1 cargo run -p vm_translator -- ../static/vm/FibonacciElemen
cat ../static/vm/FibonacciElement/FibonacciElement.hack
```

## Execute .hack (preaty printed)
```
git submodule init
cd c_code/hack_executor
make compile
cd ../../rust_code
cargo run -p hack_executor -- ../static/vm/FibonacciElement/FibonacciElement.hack
```

## TODO
1. Translate .jack to .vm => Translate .jack to .hack
2. Translate .jack to llvm format

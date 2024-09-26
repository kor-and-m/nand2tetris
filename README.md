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
TO_BINARY=1 cargo run -p vm_translator -- ../static/vm/FibonacciElement
cat ../static/vm/FibonacciElement/FibonacciElement.hack
```

## Translate .jack to .vm
```
cd rust_code
cargo run -p jack_compiler -- ../static/jack/Pong
ls ../static/jack/Pong/*.vm
```

## Translate .jack to .hack (preaty printed)
```
cd rust_code
cargo run -p jack_compiler -- ../static/jack/Pong
TO_BINARY=1 cargo run -p vm_translator -- ../static/jack/Pong
cat ../static/jack/Pong/Pong.hack
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
1. Translate .jack to llvm format

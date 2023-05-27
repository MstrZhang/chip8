# chip8 emulator

## overview

this is a chip8 emulator written in rust to explore wasm, writing code in rust, and emulation in general. code comes from [the following book](https://github.com/aquova/chip8-book). would highly recommend for anyone interested in any of these topics. you don't need any prior knowledge of rust but you should have a high-level understanding of operating systems and programming in general

## build

**for desktop**

```bash
$ cd desktop
$ cargo run <PATH_TO_ROM>
```

**for web**

```bash
$ cd wasm
$ wasm-pack build --target web
$ mv pkg/wasm_bg.wasm ../web
$ mv pkg/wasm.js ../web
```

run from `index.html`

## controls

keys are oriented in a grid like how some chip-8 games expect

|keyboard|chip-8|
|---|---|
|1|1|
|2|2|
|3|3|
|4|C|
|Q|4|
|W|5|
|E|6|
|R|D|
|A|7|
|S|8|
|D|9|
|F|E|
|Z|A|
|X|0|
|C|B|
|V|F|

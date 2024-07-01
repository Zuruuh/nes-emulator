Very WIP NES Emulator. The core engine is built with rust, compiled into web assembly, then embedded in a webview with Tauri.

TODO list:
- make it work correctly lol
- publish to github pages or something like that
- add some more tests
- find some more resources on the CPU emulator
- add debugging tools (something to view/dump memory changes in real time ? slow down emulation ? or read opcodes step by step ?) bcz it's a pain rn

## Requirements

- tauri 2.0 dependencies (`nix develop` or
  https://beta.tauri.app/start/prerequisites/)
- cargo-tauri (`cargo install tauri-cli --version '^2.0.0-beta`)

## Run

```shell
cargo tauri dev
```

## References
- https://www.nesdev.org/obelisk-6502-guide/reference.html | ASM6502 reference. This is the format NES games are compiled in.

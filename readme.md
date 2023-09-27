# bevy_cosmic_edit

This bevy plugin provides multiline text editing for bevy apps, thanks to [cosmic_text](https://github.com/pop-os/cosmic-text) crate!

Emoji, ligatures, and other fancy stuff is supported!

![bevy_cosmic_edit](./bevy_cosmic_edit.png)

## Usage

⚠️ *Warning: This plugin is currently in early development, and its API is subject to change.*

Explore examples folder for basic usage.

Native:

```rust
cargo r --example text_input
```

Wasm:

```rust
cargo install wasm-server-runner
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo r --target wasm32-unknown-unknown --example restricted_input
```

## Compatibility

| bevy | bevy_cosmic_edit |
| ---- | ---------------- |
| 0.11 | 0.8 - latest     |


## License

MIT or Apache-2.0

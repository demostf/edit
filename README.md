# Demo editor

## Rust api

```rust
use edit::{edit, EditOptions, TickRange};

fn main() {
    let options = EditOptions {
        unlock_pov: true,
        cut: Some(TickRange {
            from: 1000.into(),
            to: 2000.into(),
        }),
        ..EditOptions::default()
    };
    let input = fs::read("in.demo").unwrap();
    let output = edit(&input, options);
    fs::write("out.dem", output).unwrap();
}
```
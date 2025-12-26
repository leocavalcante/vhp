---
layout: default
title: Installation
nav_order: 3
---

# Installation

VHP is built entirely in Rust with zero external dependencies.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later recommended)

## Build from Source

```bash
git clone https://github.com/leocavalcante/vhp.git
cd vhp
cargo build --release
```

The binary will be at `./target/release/vhp`

## Add to PATH (Optional)

To use `vhp` from anywhere:

```bash
# Linux/macOS
sudo cp ./target/release/vhp /usr/local/bin/

# Or add to your shell profile
export PATH="$PATH:/path/to/vhp/target/release"
```

## Run Directly with Cargo

You can also run VHP directly through Cargo without installing:

```bash
cargo run --release -- script.php
cargo run --release -- -r 'echo "Hello!";'
```

## Verify Installation

```bash
vhp --help
```

You should see the help output with available options.

<h1 align="center">
  <br>
  <img src="https://raw.githubusercontent.com/leocavalcante/vhp/main/docs/assets/logo.png" alt="VHP Logo" width="200">
  <br>
  VHP
  <br>
</h1>

<h4 align="center">Vibe-coded Hypertext Preprocessor</h4>

<p align="center">
  <em>A PHP superset built entirely in Rust through AI-assisted development</em>
</p>

<p align="center">
  <a href="https://leocavalcante.github.io/vhp/features">Features</a> •
  <a href="https://leocavalcante.github.io/vhp/installation">Installation</a> •
  <a href="https://leocavalcante.github.io/vhp/usage">Usage</a> •
  <a href="https://leocavalcante.github.io/vhp/examples">Examples</a> •
  <a href="https://leocavalcante.github.io/vhp/roadmap">Roadmap</a>
</p>

---

## What is VHP?

**VHP** is a modern PHP implementation written from scratch in Rust. The name stands for "**V**ibe-coded **H**ypertext **P**reprocessor" — reflecting that it's being built entirely through prompts to AI agents ("vibe coding").

### Goals

- **Fast** — Native performance via Rust compilation
- **Secure** — Memory safety guaranteed by Rust's ownership model
- **Zero Dependencies** — Built using only Rust's standard library
- **PHP 8.x Compatible** — Run existing PHP code with zero modifications
- **Progressive** — New features added incrementally with comprehensive tests

## Quick Start

```bash
# Build
git clone https://github.com/leocavalcante/vhp.git
cd vhp
cargo build --release

# Run a file
./target/release/vhp script.php

# Run inline code
./target/release/vhp -r 'echo "Hello, VHP!";'
```

## Features at a Glance

- ✅ PHP tags (`<?php`, `?>`, `<?=`)
- ✅ Variables, operators, and expressions
- ✅ Control flow (`if`/`else`, `while`, `for`, `switch`)
- ✅ User-defined and recursive functions
- ✅ 50+ built-in functions (string, math, type, output)
- ✅ PHP-compatible type coercion
- ✅ Mixed HTML/PHP support

See the [full features documentation](https://leocavalcante.github.io/vhp/features) for details.

## Why "Vibe Coding"?

VHP is an experiment in AI-assisted software development. Every line of code has been written through conversations with AI agents (Claude). The goal is to demonstrate that complex systems like programming language interpreters can be built entirely through natural language prompts.

## Contributing

Contributions are welcome! Feel free to:
- Open issues for bugs or feature requests
- Submit pull requests
- Improve documentation
- Add more tests

## License

BSD 3-Clause License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with Rust and AI
</p>

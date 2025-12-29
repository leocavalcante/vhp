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
  <a href="https://github.com/leocavalcante/vhp/actions/workflows/ci.yml"><img src="https://github.com/leocavalcante/vhp/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/leocavalcante/vhp/blob/main/LICENSE"><img src="https://img.shields.io/github/license/leocavalcante/vhp" alt="License"></a>
  <a href="https://github.com/leocavalcante/vhp"><img src="https://img.shields.io/github/stars/leocavalcante/vhp" alt="Stars"></a>
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
- ✅ Control flow (`if`/`else`, `while`, `for`, `foreach`, `switch`)
- ✅ Arrays (indexed, associative, nested)
- ✅ User-defined and recursive functions
- ✅ Classes & Objects (properties, methods, constructors, `$this`, static calls, inheritance)
- ✅ Interfaces and Traits
- ✅ Constructor Property Promotion (PHP 8.0)
- ✅ Match expressions (PHP 8.0)
- ✅ Named arguments (PHP 8.0)
- ✅ 65+ built-in functions (string, math, array, type, output)
- ✅ PHP-compatible type coercion
- ✅ Mixed HTML/PHP support

See the [full features documentation](https://leocavalcante.github.io/vhp/features) for details.

## Why "Vibe Coding"?

VHP is an experiment in AI-assisted software development. Every line of code has been written through conversations with AI agents (Claude). The goal is to demonstrate that complex systems like programming language interpreters can be built entirely through natural language prompts.

## Why VHP Instead of Just Vibe Coding Rust?

You might wonder: "If AI can write code, why not just vibe code your project directly in Rust?"

The answer is **existing codebases**. One of VHP's primary goals is to run existing PHP code with zero modifications. There are millions of PHP applications in production today — WordPress, Laravel, Drupal, and countless custom systems. VHP aims to provide a fast, secure runtime for all of them without requiring developers to rewrite their code.

Think of it this way: vibe coding Rust gets you a new application. VHP gets you a new runtime for *all* PHP applications.

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

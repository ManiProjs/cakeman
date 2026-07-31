# Cakeman

A no-nonsense C/C++ package manager

## About

This project was originally written in Go, but because Go is a _terrible_ language (anyone says that), we are rewritting it in **Rust**.

## Installation

Currently, there are no instructions for installation.

## Build from source

Install Rust and a C/C++ compiler.

Run:

```shell
cargo run
```

If you want to run a release binary, run this:

```shell
cargo run --release
```

## Features

**Note:** "No" doesn't mean that it won't be available in the future. It may be available in the future.

| Feature               | Supported OS/OSes | Available | It is working? |
|-----------------------|-------------------|-----------|----------------|
| Supports dependencies | Cross-platform    | ✅ Yes     | ❌ Not recursive          |
| Have a build system   | N/A               | ❌ No      | N/A            |
| Install C tools       | N/A               | ❌ No      | N/A            |

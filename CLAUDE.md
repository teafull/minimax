# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MiniMax is a Rust SDK for MiniMax's LLM API services with OpenAI and Anthropic API compatibility. The project uses a Rust workspace with three crates.

## Build & Test Commands

```bash
cargo build --workspace        # Build entire workspace
cargo test --workspace         # Run all tests
cargo test -p <crate>         # Run tests for specific crate
cargo check --workspace        # Check compilation
cargo clippy --workspace      # Run linting
cargo run -p minimax           # Run the test program
```

## Architecture

**Workspace structure:**
- `crates/openai/` - OpenAI-compatible API library (Bearer token auth)
- `crates/anthropic/` - Anthropic-compatible API library (X-Api-Key header)
- `crates/minimax/` - Test program that uses both interfaces

**Key patterns:**
- Builder pattern for API requests (`ChatBuilder`)
- `Client` / `AnthropicClient` structs own the HTTP client and API key
- `thiserror` for error handling with `Error` enum and `Result<T>` type

**API endpoints:**
- OpenAI: `https://api.minimaxi.com/v1/chat/completions`
- Anthropic: `https://api.minimaxi.com/anthropic/v1/messages`

## Dependencies

- `reqwest` (blocking) - HTTP client
- `serde` / `serde_json` - JSON serialization
- `thiserror` - Error handling

## Running Tests

Set API key and run:
```bash
MINIMAX_API_KEY=your-key cargo run -p minimax
```

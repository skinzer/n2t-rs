# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Build and Test
```bash
# Build the entire project
cargo build

# Run all tests
cargo test

# Run specific test modules
cargo test --lib chip::builder           # Test chip builder functionality
cargo test --lib sequential             # Test sequential chip components
cargo test --lib computer               # Test computer-level components
cargo test rom32k                       # Test specific chip implementation

# Run with output for debugging
cargo test -- --nocapture

# Check code without building
cargo check

# Run the CLI (when implemented)
cargo run --bin n2t_cli
```

### Development
```bash
# Format code
cargo fmt

# Lint code (if clippy is configured)
cargo clippy

# Generate documentation
cargo doc --open
```

## Architecture Overview

This is a Rust translation of the nand2tetris hardware simulator, implementing a complete digital circuit simulation system. The architecture follows a layered approach:

### Core Simulation Engine (`src/chip/`)

**Pin and Bus System**: The foundation uses `Pin` trait objects connected via `Bus` implementations. Buses support both single-bit and multi-bit (up to 16-bit) operations with individual bit access.

**Chip Interface**: All chips implement `ChipInterface` providing standardized pin access, evaluation, and reset capabilities. Chips can be either:
- **Combinatorial**: Immediate output calculation via `eval()`
- **Sequential**: Clock-driven state changes via `ClockedChip` trait

**Memory Management**: Uses `Rc<RefCell<dyn Pin>>` for shared mutable pin references, allowing multiple connections to the same pin while maintaining Rust's safety guarantees.

### Chip Hierarchy (`src/chip/builtins/`)

Chips are organized into categories reflecting the nand2tetris curriculum:

- **`logic/`**: Basic gates (NAND, NOT, AND, OR, XOR, MUX, DMUX)
- **`arithmetic/`**: 16-bit operations (Add16, Inc16, ALU, Adders)  
- **`sequential/`**: Stateful chips (DFF, Bit, Register, PC, RAM hierarchy)
- **`computer/`**: System components (ROM32K, Screen, Keyboard)

**RAM Hierarchy**: Complete memory system from RAM8 (8 registers) to RAM16K (16384 registers), each with proper address masking and clocked behavior.

**Memory-Mapped I/O**: Screen (8192 words at offset 16384) and Keyboard (single word at offset 24576) following the Hack computer specification.

### Language Parsing (`src/languages/`)

**HDL Parser**: Converts Hardware Description Language to chip definitions. Supports pin declarations with bit widths, part instantiation, and wire connections including pin ranges (e.g., `a[0..7]`).

**Test Framework**: Implements `.tst` file parsing for automated chip testing with cycle-accurate simulation.

### Key Design Patterns

**Builder Pattern**: `ChipBuilder` maintains a registry of builtin chips and constructs chip networks from HDL specifications.

**Clock Synchronization**: `Clock` system uses `tokio::broadcast` channels for coordinating sequential logic across multiple chips.

**SubBus System**: Enables pin range connections (e.g., connecting `out[0..7]` to `in[8..15]`) with proper bit mapping and width validation.

**Error Handling**: Comprehensive `Result`-based error propagation using `thiserror` for structured error types.

## Testing Strategy

Tests are organized by component level:
- **Unit tests**: Within each chip module (`mod tests`)
- **Integration tests**: Cross-component functionality (`src/chip/tests.rs`, `src/lib.rs`)
- **Sequential tests**: Clock-driven behavior validation (`src/chip/sequential_tests.rs`)
- **Framework tests**: Testing infrastructure validation (`src/test/`)

The test suite covers 108+ test cases ensuring correctness of the entire simulation pipeline from HDL parsing to chip evaluation.

## Development Context

This project translates the TypeScript nand2tetris simulator to Rust while preserving the original's architecture and behavior. Key translation patterns:

- **Shared State**: `Rc<RefCell<>>` replaces JavaScript's mutable references
- **Async Coordination**: `tokio::broadcast` replaces RxJS observables  
- **Dynamic Typing**: Rust traits provide type safety while maintaining flexibility
- **Memory Safety**: Rust's ownership system eliminates potential memory errors from the original

Current implementation status: Complete through computer-level components, ready for CPU and VM implementation phases.
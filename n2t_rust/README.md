# nand2tetris Rust Simulator

A Rust implementation of the nand2tetris hardware and software simulator, translated from the original TypeScript version.

## Project Structure

```
src/
├── lib.rs              # Main library entry point
├── bin/
│   └── cli.rs          # Command line interface
├── chip/               # Digital circuit simulation
│   ├── mod.rs          # Module exports
│   ├── pin.rs          # Pin and voltage definitions
│   ├── bus.rs          # Bus implementation with connections
│   ├── clock.rs        # Clock synchronization system
│   └── chip.rs         # Chip container and evaluation
├── cpu/                # CPU emulation (placeholder)
├── vm/                 # Virtual machine (placeholder)
├── languages/          # Language parsers (placeholder)
├── test/               # Testing framework (placeholder)
└── error.rs            # Error types and handling
```

## Current Status

✅ **Phase 1 Core Structures**: Basic pin/bus/chip architecture
🔄 **Phase 2**: Language parsers (HDL first)
⏳ **Phase 3**: CPU and VM implementation
⏳ **Phase 4**: Testing framework
⏳ **Phase 5**: Async integration

## Building

```bash
cargo build
cargo test
cargo run --bin n2t_cli
```

## Design Principles

- **Synchronous First**: Deterministic behavior before async complexity
- **Memory Safety**: Leveraging Rust's ownership system
- **Error Handling**: Comprehensive Result-based error propagation
- **Modular**: Clear separation of concerns matching original architecture

## Translation Notes

This implementation maintains the core architecture of the original TypeScript simulator while adapting to Rust idioms:

- RxJS observables → `tokio::sync::broadcast` channels
- Shared mutable state → `Rc<RefCell<>>` patterns
- Dynamic typing → Strong static typing with traits
- Promise-based async → Rust async/await with tokio
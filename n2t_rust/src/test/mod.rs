// Testing framework module

pub mod chiptst;
pub mod runner;
pub mod comparator;
pub mod harness;

#[cfg(test)]
mod chiptst_tests;

pub use chiptst::{ChipTest, OutputSpec, TestInstruction, TestSetInstruction, TestEvalInstruction, TestOutputInstruction, TestTickInstruction, TestTockInstruction, TestCompoundInstruction};
pub use runner::TestRunner;
pub use comparator::TestComparator;
pub use harness::TestHarness;
//! Collection of optimisations on `Operation`s.

mod merge_offsets;
pub use merge_offsets::merge_offsets;
mod reorder_operations;
pub use reorder_operations::reorder_operations;
mod remove_offsets_before_zeroing;
pub use remove_offsets_before_zeroing::remove_offsets_before_zeroing;
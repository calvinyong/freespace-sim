#![forbid(unsafe_code)]
pub mod buddy;
pub mod demos;
pub mod freelist;
pub mod prelude;
pub mod workloads;

#[derive(Debug, Copy, Clone)]
pub enum Policy {
    Best,
    First,
}

pub trait Allocator {
    /// Allocate memory for the requested size. Returns None
    /// if space cannot be allocated
    fn malloc(&mut self, size: usize) -> Option<usize>;

    /// Frees the memory for the given pointer. Returns
    /// an error if the pointer doesn't exist
    fn free(&mut self, ptr: usize) -> Result<(), &str>;

    /// Get the the largest amount of memory that is
    /// possible to allocate
    fn largest_alloc(&self) -> usize;

    /// Get the total free space, which might not be possible
    /// to request due to external fragmentation
    fn free_space(&self) -> usize;

    /// Get the amount of internal fragmentation
    fn internal_frag(&self) -> usize;

    /// If there is free space, get a measure
    /// of the external fragmentation
    fn external_frag(&self) -> f32 {
        1.0 - (self.largest_alloc() as f32 / self.free_space() as f32)
    }

    /// Print the allocator. Too lazy to implement Display
    fn print(&self);
}

// A Bin with a capacity measured as u64.
pub trait PartitionBin {
    type Size;

    // try to fill the bin with a resource of certain size, 
    // return true on succeed, false on failure
    fn fill(&mut self, resource_size: Self::Size) -> bool;
}
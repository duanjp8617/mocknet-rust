// A Bin with a capacity measured as u64.
pub trait PartitionBin {
    type Size;
    type Id;

    // try to fill the bin with a resource of certain size, 
    // return true on succeed, false on failure
    fn fill(&mut self, resource_size: Self::Size) -> bool;

    // get the Id of this bin
    fn bin_id(&self) -> Self::Id;
}
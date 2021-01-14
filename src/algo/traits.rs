use std::collections::HashMap;

/// Implementor is used as a bin for storing items.
pub trait PartitionBin {
    type Size;
    type BinId;

    /// Try to fill the bin with an item of a certain size.
    /// 
    /// Return true on succeed, false on failure
    fn fill(&mut self, item_size: Self::Size) -> bool;

    /// Get the id of this bin.
    fn bin_id(&self) -> Self::BinId;
}
pub trait Weighted {
    fn get_weight(&self) -> usize;
}

/// Implementor stores multiple items for partition.
pub trait Partition<'a, T, I>
where
    T: 'a + PartitionBin,
    I: Iterator<Item = &'a mut T>
{
    type ItemId;

    /// Partition the stored items into bins.
    /// 
    /// Return the mapping from the item id to bin id.
    fn partition(&self, bins: I, partition_number: usize, rank_swap: bool, rank_swap_mode: String, cluster_threshold: usize) -> Result::<HashMap<Self::ItemId, <T as PartitionBin>::BinId>, String>;
    //fn partition(&self, bins: I) -> Result<HashMap<Self::ItemId, <T as PartitionBin>::BinId>, String>;
}
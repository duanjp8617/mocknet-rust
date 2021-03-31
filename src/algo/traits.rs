use std::collections::HashMap;

/// Implementor is used as a bin for storing items.
pub trait PartitionBin {
    type Size;
    type BinId;

    /// Try to fill the bin with an item of a certain size.
    ///
    /// Return true on succeed, false on failure
    fn fill(&mut self, item_size: Self::Size) -> bool;

    /// Try to release an item of a certain size from the bin
    ///
    /// Return true on succeed, false on failure
    fn release(&mut self, item_size: Self::Size) -> bool;

    /// Get the id of this bin.
    fn bin_id(&self) -> Self::BinId;
}

/// Implementor stores multiple items for partition.
pub trait Partition<'a, T, I>
where
    T: 'a + PartitionBin,
    I: Iterator<Item = &'a mut T>,
{
    type ItemId;

    /// Partition the stored items into bins.
    ///
    /// Return the mapping from the item id to bin id.
    fn partition(&self, bins: I) -> Option<HashMap<Self::ItemId, <T as PartitionBin>::BinId>>;
}

pub trait Min {
    fn minimum() -> Self;
}

pub trait Max {
    fn maximum() -> Self;
}

impl Min for u64 {
    fn minimum() -> Self {
        u64::MIN
    }
}

impl Max for u64 {
    fn maximum() -> Self {
        u64::MAX
    }
}
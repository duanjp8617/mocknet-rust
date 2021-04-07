use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

const TOTAL_ADDRS: usize = 16646144;

#[derive(Serialize, Deserialize)]
pub(crate) struct Ipv4AddrAllocator {
    ipv4_base: [u8; 4],
    curr_idx: u32,
    counter: usize,
}

impl Ipv4AddrAllocator {
    pub(crate) fn new() -> Self {
        Self {
            ipv4_base: [10, 0, 0, 0],
            curr_idx: 1,
            counter: 0,
        }
    }

    pub(crate) fn try_alloc(&mut self) -> Option<Ipv4Addr> {
        if self.counter >= TOTAL_ADDRS {
            return None
        }

        let base_addr: Ipv4Addr = self.ipv4_base.into();
        let base_addr_u32: u32 = base_addr.into();

        loop {
            let new_addr: Ipv4Addr = (base_addr_u32 + self.curr_idx).into();
            let new_addr_array = new_addr.octets();
            self.curr_idx += 1;

            if new_addr_array[3] != 0 && new_addr_array[3] != 255 {
                self.counter += 1;
                break Some(new_addr);
            }
        }
    }

    pub(crate) fn remaining_addrs(&self) -> usize {
        TOTAL_ADDRS - self.counter
    }

    pub(crate) fn reset(&mut self) {
        self.curr_idx = 1;
        self.counter = 0;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_valid() {
        let mut allocator = Ipv4AddrAllocator::new();
        let mut prev_addr = allocator.try_alloc().unwrap();
        assert_eq!(prev_addr.octets()[3] != 0, true);
        assert_eq!(prev_addr.octets()[3] != 255, true);
        assert_eq!(prev_addr.octets()[0] == 10, true);

        while let Some(curr_addr) = allocator.try_alloc() {
            assert_eq!(curr_addr.octets()[3] != 0, true);
            assert_eq!(curr_addr.octets()[3] != 255, true);
            assert_eq!(curr_addr.octets()[0] == 10, true);
            assert_eq!(curr_addr > prev_addr, true);

            prev_addr = curr_addr;
        }
    }

    #[test] 
    fn remaining() {
        let mut allocator = Ipv4AddrAllocator::new();
        allocator.try_alloc().unwrap();
        allocator.try_alloc().unwrap();
        assert_eq!(allocator.remaining_addrs(), TOTAL_ADDRS - 2);
    }
}

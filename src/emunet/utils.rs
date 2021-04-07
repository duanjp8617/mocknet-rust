use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

fn least_sigbit_idx(n: u32) -> u32 {
    let mut idx = 0;
    let mut mask = 1;

    while n & mask == 0 {
        idx += 1;
        mask = 1 << idx;
    }

    idx
}

// allocate a unique IPv4 subnet with at least 4 addresses
// this means that the subnet mask should be 0-30
#[derive(Serialize, Deserialize)]
pub(crate) struct SubnetAllocator {
    base: [u8; 4],
    subnet_len: u32,
    total_subnets: u32,
    curr_idx: u32,
}

impl SubnetAllocator {
    pub(crate) fn new(base: [u8; 4], subnet_len: u32) -> Self {
        assert!(subnet_len <= 30);

        let base_u32: u32 = Ipv4Addr::from(base).into();
        assert!(least_sigbit_idx(base_u32).min(24) > 32 - subnet_len);

        let total_subnets: u32 = ((2 as u32).pow(least_sigbit_idx(base_u32).min(24))
            - (2 as u32).pow(32 - subnet_len))
            >> (32 - subnet_len);

        Self {
            base,
            subnet_len,
            total_subnets: total_subnets + 1,
            curr_idx: 0,
        }
    }

    pub(crate) fn try_alloc(&mut self) -> Option<(u32, u32)> {
        if self.curr_idx == self.total_subnets {
            None
        } else {
            let base_u32: u32 = Ipv4Addr::from(self.base).into();
            let subnet = base_u32 + (self.curr_idx << (32 - self.subnet_len));

            self.curr_idx += 1;

            Some((subnet, self.subnet_len))
        }
    }

    pub(crate) fn remaining_subnets(&self) -> usize {
        (self.total_subnets - self.curr_idx) as usize
    }

    pub(crate) fn reset(&mut self) {
        self.curr_idx = 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_least_sigbit_bit() {
        assert_eq!(least_sigbit_idx(1), 0);
        assert_eq!(least_sigbit_idx(2), 1);
        assert_eq!(least_sigbit_idx(3), 0);
        assert_eq!(least_sigbit_idx((1 as u32) << 24), 24);
        assert_eq!(least_sigbit_idx((1 as u32) << 30), 30);
    }

    #[test]
    #[should_panic]
    fn large_subnet_len() {
        let _ = SubnetAllocator::new([10, 0, 0, 0], 31);
    }

    #[test]
    #[should_panic]
    fn invalid_base1() {
        let _ = SubnetAllocator::new([1, 0, 0, 0], 8);
    }

    #[test]
    #[should_panic]
    fn invalid_base2() {
        let _ = SubnetAllocator::new([1, 0, 0, 0], 7);
    }

    #[test]
    fn number_of_subnets() {
        let a = SubnetAllocator::new([10, 0, 0, 0], 9);
        assert_eq!(a.remaining_subnets(), 2);

        let a = SubnetAllocator::new([10, 0, 0, 0], 10);
        assert_eq!(a.remaining_subnets(), 4);

        let a = SubnetAllocator::new([10, 0, 0, 0], 24);
        assert_eq!(a.remaining_subnets(), (2 as usize).pow(24 - 8));
    }

    #[test]
    fn all_subnets() {
        let mut a = SubnetAllocator::new([10, 1, 0, 0], 24);

        while let Some(res) = a.try_alloc() {
            let ipv4 = Ipv4Addr::from(res.0);
            println!("{}/{}", ipv4, res.1);
        }
    }
}

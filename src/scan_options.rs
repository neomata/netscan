use std::{net::{Ipv4Addr, Ipv6Addr}, fmt::Debug};

pub struct ScanOptions {
    pub pool: usize,
    pub wait: f64,
    pub prefix: Option<u8>,
    pub subnet_v4: Option<Ipv4Addr>,
    pub subnet_v6: Option<Ipv6Addr>,
}

impl Debug for ScanOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScanOptions")
            .field("pool", &self.pool)
            .field("wait", &self.wait)
            .field("prefix", &self.prefix)
            .field("subnet_v4", &self.subnet_v4)
            .field("subnet_v6", &self.subnet_v6)
            .finish()
    }
}

impl Copy for ScanOptions {

}

impl Clone for ScanOptions {
    fn clone(&self) -> Self {
       ScanOptions {
            pool: self.pool,
            wait: self.wait,
            prefix: self.prefix,
            subnet_v4: self.subnet_v4,
            subnet_v6: self.subnet_v6,
        } 
    }
}
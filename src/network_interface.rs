use std::fmt::Display;
use std::net::IpAddr;
use std::net::{Ipv4Addr, Ipv6Addr};

struct NetworkInterface {
    name: String,
    ipv4: Vec<Ipv4Addr>,
    ipv6: Vec<Ipv6Addr>,
    prefix: Option<u32>,
    gateway: Option<IpAddr>,
    running: bool
}

impl Clone for NetworkInterface {
    fn clone(&self) -> Self {
        NetworkInterface { 
           name: self.name.clone(),
           ipv4: self.ipv4.clone(),
           ipv6: self.ipv6.clone(),
           prefix: self.prefix,
           gateway: self.gateway,
           running: self.running
        }
    }
}



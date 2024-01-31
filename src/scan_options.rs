use std::{net::{IpAddr, Ipv4Addr, Ipv6Addr}, fmt::Debug};
use std::collections::VecDeque;
use either::Either;

pub struct ScanOptions {
    pub ip_address: Either<Ipv4Addr, Ipv6Addr>,
    pub pool: usize,
    pub wait: f64,
    pub prefix: Option<u8>,
    pub subnet_v4: Option<Ipv4Addr>,
    pub subnet_v6: Option<Ipv6Addr>,
}

impl ScanOptions {
    fn new() -> Self {
        return ScanOptions::default()
    }

    fn organize_parameters(params: Vec<String>) -> (Either<Ipv4Addr, Ipv6Addr>, Vec<Vec<String>>) {
        // the IP Address provided should exist as either the first or last argument, not in the middle
        let mut pairs_last: VecDeque<Vec<String>> = params.clone().chunks(2).map(|s| s.to_vec()).collect();
        let mut pairs_first: VecDeque<Vec<String>> = params.clone().rchunks(2).map(|s| s.to_vec()).collect();

        let mut pairs_choice = Vec::from_iter([pairs_last, pairs_first])
            .into_iter()
            .map(|pairs| {
                let found_ip = pairs.back()
                    .unwrap()
                    .first()
                    .unwrap()
                    .contains(".") 
                ||
                pairs.back()
                    .unwrap()
                    .first()
                    .unwrap()
                    .contains(":");
                (pairs, found_ip)
            })
            .find(|pf| pf.1)
            .map(|pf| pf.0)
            .unwrap_or_else(|| {
                println!("IP address must be either first or last argument in list");
                std::process::exit(4);
            });

        let ip = pairs_choice.pop_back()
            .map(|i| i.first().unwrap().clone().to_owned())
            .unwrap_or_else(|| String::new());
        let ipv4_ior_ipv6: Either<Ipv4Addr, Ipv6Addr> = if let Ok(ip_address) = ip.parse::<Ipv4Addr>() {
            Either::Left(ip_address)
        } else if let Ok(ip_address) = ip.parse::<Ipv6Addr>() {
            Either::Right(ip_address)
        } else {
            println!("Unformatted IP Address provided");
            std::process::exit(4)
        };
        (ipv4_ior_ipv6, pairs_choice.into())
    }

    pub fn from_arguments(arguments: Vec<String>) -> ScanOptions {
        let mut options = ScanOptions::new();
                
        let parameters: Vec<String> = arguments.clone(); 

        let (ip_address, parameter_pairs) = ScanOptions::organize_parameters(parameters.clone());
        options.ip_address = ip_address;
        if parameters.is_empty() { 
            println!("Requires IP address");
            std::process::exit(4);
        }

        for pair in parameter_pairs {
            let slice = &pair[..];
            match slice {
                [key, val] if key == "--pool" || key == "-P" => if let Ok(pool) = val.parse::<usize>() {
                    options.pool = pool;
                } else {
                    println!("Thread pool size needs to be a positive whole number");
                    std::process::exit(4);
                },

                [key, val] if key == "--wait" || key == "-w" => if let Ok(wait) = val.parse::<f64>() {
                    options.wait = wait;
                } else {
                    println!("Wait time (ms) needs to be a positive number");
                    std::process::exit(4);
                },

                [key, val] if key == "--prefix" || key == "-p" => if let Ok(prefix) = val.parse::<u8>() {
                    options.prefix = Some(prefix);
                } else {
                    println!("Prefix needs to be a number [0, 32] inclusive");
                    std::process::exit(4);
                },

                [key, val] if key == "--subnet" || key == "-s" => if let Ok(subnet) = val.parse::<Ipv4Addr>() {
                    options.subnet_v4 = Some(subnet);
                } else if let Ok(subnet) = val.parse::<Ipv6Addr>() { 
                    options.subnet_v6 = Some(subnet);
                } else {
                    println!("Subnet needs to be a valid IP Address");
                    std::process::exit(4);
                },

                [key, val] => {
                    println!("Unrecognized argument: {}", key);
                    std::process::exit(4);
                },

                _ => {
                    println!("Unrecognized argument");
                    std::process::exit(4);
                }
            }
        }
        
        return options;
    }
}

impl Default for ScanOptions {
    fn default() -> Self {
        return ScanOptions {
            ip_address: Either::Left(Ipv4Addr::LOCALHOST),
            pool: 256,
            wait: 1000.0,
            prefix: None,
            subnet_v4: None,
            subnet_v6: None,
        }
    }
}

impl Debug for ScanOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScanOptions")
            .field("ip_address", &self.ip_address)
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
            ip_address: self.ip_address,
            pool: self.pool,
            wait: self.wait,
            prefix: self.prefix,
            subnet_v4: self.subnet_v4,
            subnet_v6: self.subnet_v6,
        } 
    }
}
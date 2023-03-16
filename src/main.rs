extern crate default_net;
extern crate ipnetwork;
extern crate futures;

mod scan_options;

use futures::{StreamExt, SinkExt};
use ipnetwork::{Ipv4Network, Ipv6Network};
use std::collections::{HashMap, HashSet};
use std::env::args;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Div;
use std::env::consts;
use std::process::{Command};
use futures::executor::ThreadPool;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use scan_options::ScanOptions;

fn main() {

    let mut options = ScanOptions {
        pool: 256,
        wait: 1000.0,
        prefix: None,
        subnet_v4: None,
        subnet_v6: None,
    };

    // skipping 1 because the executable name is not relevant
    let arguments: Vec<String> = args().skip(1).collect();
    fn parse_arguments(arguments: Vec<String>, options: &mut ScanOptions) -> String {
        let parameters = arguments.clone();
        let parameter_pairs = parameters.chunks(2);
        if parameters.is_empty() { panic!("Requires IP address"); }

        let keys = arguments.chunks(2).map(|c| c.first()).collect::<Vec<_>>();
        let mut key_set: HashSet<Option<&String>> = HashSet::with_capacity(4);
        for key in keys {
            if !key_set.contains(&key) {
                key_set.insert(key);
            } else {
                panic!("Define optional parameters once")
            }
        }

        for pair in parameter_pairs {
            match pair {
                [key, val] if key == "-pool" => if let Ok(pool) = val.parse::<usize>() {
                    options.pool = pool;
                } else {
                    panic!("Thread pool size needs to be a positive whole number");
                },
                [key, val] if key == "-wait" => if let Ok(wait) = val.parse::<f64>() {
                    options.wait = wait;
                } else {
                    panic!("Wait time (ms) needs to be a positive number");
                },
                [key, val] if key == "-prefix" => if let Ok(prefix) = val.parse::<u8>() {
                    options.prefix = Some(prefix);
                } else {
                    panic!("Prefix needs to be a number [0, 32] inclusive");
                },
                [key, val] if key == "-subnet" => if let Ok(subnet) = val.parse::<Ipv4Addr>() {
                    options.subnet_v4 = Some(subnet);
                } else if let Ok(subnet) = val.parse::<Ipv6Addr>() { 
                    options.subnet_v6 = Some(subnet);
                } else {
                    panic!("Subnet needs to be a valid IP Address");
                },
                [ip] => { return ip.to_string(); },
                _ => panic!("Unrecognized optional parameter")
            }
        }
        
        panic!("Netscan can only accept one IP address");
    } 

    fn peers(input_ip: String, options: ScanOptions) -> Vec<String> {
        let pool = ThreadPool::builder().pool_size(options.pool).create().expect("Unable to create thread pool");
        let channel = unbounded::<String>();
        let mut sender = channel.0;
        let mut receiver = channel.1;
        let raw_interfaces = default_net::get_interfaces();
        let mut ipv4_networks: HashMap<Ipv4Addr, Ipv4Network> = HashMap::new();
        let mut ipv6_networks: HashMap<Ipv6Addr, Ipv6Network> = HashMap::new();
        for interface in raw_interfaces {
            for ip in &interface.ipv4[..] {
                let network = ipnetwork::Ipv4Network::new(ip.addr, ip.prefix_len).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip.addr, ip.prefix_len));
                ipv4_networks.insert(ip.addr, network);
            }

            for ip in &interface.ipv6[..] {
                let network = ipnetwork::Ipv6Network::new(ip.addr, ip.prefix_len).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip.addr, ip.prefix_len));
                ipv6_networks.insert(ip.addr, network);
            }
        }

        let ipv4 = input_ip.parse::<Ipv4Addr>();
        let ipv6 = input_ip.parse::<Ipv6Addr>();

        let mut reach: Vec<String> = Vec::with_capacity(256);

        if let Ok(ip) = ipv4 {
            match ipv4_networks.get(&ip) {
                Some(network) => {
                    let processor = async {
                        network.iter().map(|host| ping_command(host, options, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        std::mem::drop(sender);
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    futures::executor::block_on(processor);
                },               
                None if options.prefix.is_some() => {
                    let network = Ipv4Network::new(ip, options.prefix.unwrap()).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip, options.prefix.unwrap()));
                    let processor = async {
                        network.iter().map(|host| ping_command(host, options, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        std::mem::drop(sender);
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    futures::executor::block_on(processor);
                },
                None if options.subnet_v4.is_some() => {
                    let prefix = ipnetwork::ipv4_mask_to_prefix(options.subnet_v4.unwrap()).expect(&format!("Unable to acquire prefix from subnet: {}", options.subnet_v4.unwrap()));
                    let network = Ipv4Network::new(ip, prefix).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip, prefix));
                    let processor = async {
                        network.iter().map(|host| ping_command(host, options, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        std::mem::drop(sender);
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    futures::executor::block_on(processor);
                },
                _ => panic!("Machine not assigned provided IP. Please provide network prefix length or subnet mask if IP not assigned")
            }
        } else if let Ok(_ip) = ipv6 {
            ()
        } else {
            panic!("IP provided is not valid");
        }
        return reach;
    }

    async fn ping_command(host: Ipv4Addr, options: ScanOptions, sender: UnboundedSender<String>) -> () {
        let mut command = Command::new("ping");

        match consts::OS {
            "linux"   => command.arg("-c").arg("1").arg("-W").arg(options.wait.div(1000.0).to_string()),
            "macos"   => command.arg("-c").arg("1").arg("-W").arg(options.wait.to_string()),
            "windows" => command.arg("-n").arg("1").arg("-w").arg(options.wait.to_string()),
            _ => panic!("Operating system unsupported")
        };
        
        let code = command
            .arg(host.to_string())
            .output()
            .map(|p| p.status.code())
            .unwrap_or(Option::Some(-1));
        let status = code;

        if let Some(0) = status {
            println!("{}", host.to_string());
            sender.unbounded_send(host.to_string())
                .expect("Internal transmission failed");
        }
    }

    let ip = parse_arguments(arguments, &mut options);

    let reach = peers(ip, options);

    for ip in reach {
        println!("{ip}");
    }
    
}

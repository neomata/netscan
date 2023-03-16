extern crate default_net;
extern crate ipnetwork;
extern crate futures;

use futures::StreamExt;
use ipnetwork::{Ipv4Network, Ipv6Network};
use std::collections::HashMap;
use std::env::args;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::{Stdio, Command};
use futures::executor::ThreadPool;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};


fn main() {

    let arguments: Vec<String> = args().collect();
    println!("{:?}", arguments.get(1));

    fn peers(input_ip: &str, prefix: Option<u8>, subnet: Option<IpAddr>) -> Vec<String> {
        // pool size 1024
        let pool = ThreadPool::builder().pool_size(1024).create().expect("Unable to create thread pool");
        let (sender, receiver) = unbounded::<String>();
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
                        network.iter().map(|host| ping_command(host, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    println!("the last of us part i");
                    let items = futures::executor::block_on(processor);
                    let aggregation = format!("{}\n{}", ip.to_string(), items.iter().map(|h| "\t".to_string() + h).collect::<Vec<_>>().join("\n"));
                    println!("{aggregation}");
                },               
                None if prefix.is_some() => {},
                None if subnet.is_some() => {},
                _ => panic!("Machine not assigned provided IP. Please provide network prefix length or subnet mask if IP not assigned")
            }
        } else if let Ok(_ip) = ipv6 {
            ()
        } else {
            panic!("IP provided is not valid");
        }
        return reach;
    }

    async fn ping_command(host: Ipv4Addr, sender: futures::channel::mpsc::UnboundedSender<String>) -> () {
        let mut command = std::process::Command::new("ping");

        fn os_ping_args(command: &mut std::process::Command) {
            match std::env::consts::OS {
                "linux"   => command.arg("-c").arg("1").arg("-W").arg("1"),
                "macos"   => command.arg("-c").arg("1").arg("-W").arg("1000"),
                "windows" => command.arg("-n").arg("1").arg("-w").arg("1000"),
                _ => panic!("Operating system unsupported")
            };
        }
        os_ping_args(&mut command);
        let process = command
            .arg(host.to_string())
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        let status = process.status.code();

        if let Some(0) = status {
            println!("{}", host.to_string());
            sender.unbounded_send(host.to_string())
                .expect("Internal transmission failed");
        }
    }

    let reach = peers("192.168.0.162", Some(31), None);

    for ip in reach {
        println!("{ip}");
    }

    // low (network address): 32 - prefix length = dynamic bits, lsbs of ip / 2 ^ dynamic bits
    // hi  (network address): low + 2 ^ dynamic bits - 1

    // ex. 32 - 19(prefix) = 13
    // 13 / 8(1 byte) = 1.~~, ceil = 2
    // so we only want to compute for the 2nd lsbyte
    // 
    
}

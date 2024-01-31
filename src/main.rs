extern crate default_net;
extern crate ipnetwork;
extern crate futures;

mod scan_options;

use futures::{StreamExt};
use ipnetwork::{Ipv4Network, Ipv6Network};
use std::collections::{HashMap, HashSet};
use std::env::args;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Div;
use std::env::consts;
use std::process::{Command};
use futures::executor::ThreadPool;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use netscan::scan_options::ScanOptions;

fn main() {

    // skipping 1 because the executable name is not relevant
    let args: Vec<String> = args().skip(1).collect();
    let options = ScanOptions::from_arguments(args);

    fn peers(options: ScanOptions) {
        let pool = ThreadPool::builder().pool_size(options.pool).create().expect("Unable to create thread pool");
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


        let ipv4 = options.ip_address
            .left()
            .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "scan options does not contain ipv4 address"));
        let ipv6 = options.ip_address
            .right()
            .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "scan options does not contain ipv6 address"));

        if let Ok(ip) = ipv4 {
            match ipv4_networks.get(&ip) {

                /*
                 * Input IP assigned to interface, ignore optional params -prefix and -subnet
                 */ 
                Some(network) => {
                    let processor = async {
                        network.iter().map(|host| ping_host(host, options, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        std::mem::drop(sender);
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    futures::executor::block_on(processor);
                },

                /*
                 * Input IP not assigned to interface, use provided -prefix parameter
                 */
                None if options.prefix.is_some() => {
                    let network = Ipv4Network::new(ip, options.prefix.unwrap()).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip, options.prefix.unwrap()));
                    let processor = async {
                        network.iter().map(|host| ping_host(host, options, sender.clone()))
                            .for_each(|handler| pool.spawn_ok(handler));
                        
                        std::mem::drop(sender);
                        let items = receiver.collect::<Vec<_>>();
                        items.await
                    };
                    futures::executor::block_on(processor);
                },

                /*
                 * Input IP not assigned to interface, use provided -subnet parameter
                 */
                None if options.subnet_v4.is_some() => {
                    let prefix = ipnetwork::ipv4_mask_to_prefix(options.subnet_v4.unwrap()).expect(&format!("Unable to acquire prefix from subnet: {}", options.subnet_v4.unwrap()));
                    let network = Ipv4Network::new(ip, prefix).expect(&format!("Unable to acquire network from IP: {} and prefix: {}", ip, prefix));
                    let processor = async {
                        network.iter().map(|host| ping_host(host, options, sender.clone()))
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
    }

    async fn ping_host(host: Ipv4Addr, options: ScanOptions, sender: UnboundedSender<String>) -> () {
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

        if let Some(0) = code {
            println!("{}", host.to_string());
            sender.unbounded_send(host.to_string())
                .expect("Internal transmission failed");
        }
    }

    peers(options);
    
}

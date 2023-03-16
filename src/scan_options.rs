pub struct ScanOptions {
    contains_names: Vec<String>, // match these names in the list of interfaces
    ipv4: bool,                  // include IP V4 addresses
    ipv6: bool,                  // include IP V6 addresses
    loopback: bool,              // include internal loopback addresses
    local: bool,                 // include non-loopback non-special addresses
    lan: bool                    // include addresses that pair to a gateway address
}
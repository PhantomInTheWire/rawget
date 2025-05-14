use clap::Parser;
use smoltcp::phy::Medium::Ethernet;
use url::Url;
use smoltcp::phy::TunTapInterface;
use std::net::Ipv4Addr;
use std::process::exit;
use std::str::FromStr;

mod dns;
mod ethernet;
mod http;

// GET a webpage
#[derive(Parser, Debug)]
#[command(name = "rawget", about = "GET a webpage, manually")]
struct Args {
    #[arg(short, long)]
    /// The URL to fetch
    url: String,

    /// The tap device to use
    #[arg(name = "tap-device")]
    tap_device: String,

    /// The DNS server to use (default: 1.1.1.1)
    #[arg(long, default_value = "1.1.1.1")]
    dns_server: String,
}

fn main() {
    let args = Args::parse();

    let url = Url::parse(&args.url)
        .expect("error: unable to parse <url> as a URL");
    if url.scheme() != "http" {
        eprintln!("error: only HTTP protocol supported");
        return;
    }
    let tap = TunTapInterface::new(&args.tap_device, Ethernet).expect(
        "error: unable to use <tap-device> as a network interface",
    );
    let domain_name = url.host_str().expect("invalid domain name");
    let dns_server = Ipv4Addr::from_str(&args.dns_server)
        .unwrap_or_else(|_| {
            eprintln!(
                "warning: couldn't parse DNS server address '{}', using default 1.1.1.1 instead.",
                args.dns_server
            );
            Ipv4Addr::new(1, 1, 1, 1)
        });

    let addr_list = dns::resolve(&domain_name, &dns_server);
    let ip_addr: Ipv4Addr;
    match addr_list {
        Ok(ip_list) => {
            ip_addr = ip_list[0];
        }
        Err(e) => {
            eprintln!("DNS resolution failed {}", e);
            exit(1);
        }
    }
    let mac = ethernet::MacAddress::generate();

}

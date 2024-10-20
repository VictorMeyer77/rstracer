use etc::etc::host::Host;
use etc::etc::EtcReader;

fn main() {
    display(Host::read_etc_file(None).unwrap());
}

fn display(hosts: Vec<Host>) {
    println!("{0: <25} | {1: <25}", "name", "address");
    hosts.iter().for_each(|host| {
        println!("{0: <25} | {1: <25}", host.name, host.address);
    });
}

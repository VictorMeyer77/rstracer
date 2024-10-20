use etc::etc::service::Service;
use etc::etc::EtcReader;

fn main() {
    display(Service::read_etc_file(None).unwrap());
}

fn display(services: Vec<Service>) {
    println!(
        "{0: <25} | {1: <25} | {2: <25}",
        "service", "protocol", "port"
    );
    services.iter().for_each(|service| {
        println!(
            "{0: <25} | {1: <25} | {2: <25}",
            service.name, service.protocol, service.port
        );
    });
}

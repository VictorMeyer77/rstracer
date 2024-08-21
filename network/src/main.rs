use log::warn;

use network::osi;
use pcap::Device;
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::util::MacAddr;
// todo gerer device

fn main() {
    env_logger::init();

    warn!("test");

    let mut cap = Device::lookup().unwrap().unwrap().open().unwrap();

    let dev = Device::list().unwrap();

    // println!("{:?}", dev);

    let main_device = Device::lookup().unwrap().unwrap();

    // println!("{:?}", main_device);

    for d in dev {
        println!("{:?}", d);
    }

    /*

    while let Ok(packet) = cap.next_packet() {
        let data_link_packet = osi::data_link::read_packet(&packet.data).unwrap();
        println!("{:?}", data_link_packet);
        let network_packet = osi::network::read_packet(&data_link_packet).unwrap();
        println!("{:?}", network_packet);
        let transport_packet = osi::transport::read_packet(&network_packet).unwrap();
        println!("{:?}", transport_packet);
    }*/
}

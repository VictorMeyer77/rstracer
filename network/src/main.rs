use network::error::Error;
use network::osi;
use network::osi::data_link::DataLinkProtocol;
use network::osi::network::{Network, NetworkProtocol};
use pcap::Device;
use pnet::packet::dns::DnsPacket;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::icmp::IcmpPacket;
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
// todo gerer device

fn main() {
    let mut cap = Device::lookup().unwrap().unwrap().open().unwrap();

    let dev = Device::list().unwrap();

    println!("{:?}", dev);

    let main_device = Device::lookup().unwrap().unwrap();

    println!("{:?}", main_device);

    while let Ok(packet) = cap.next_packet() {
        // Layer 2: data link

        let data_link_packet = osi::data_link::read_packet(packet.data).unwrap();

        match data_link_packet.protocol {
            DataLinkProtocol::Ethernet => {
                //println!("ethernet pac! {:?}", data_link_packet.ethernet);

                let network_packet = osi::network::read_data_link_packet(&data_link_packet);

                //println!("{:?} {:?}", network_packet.ip, network_packet.protocol);

                match network_packet.protocol {
                    NetworkProtocol::Arp => {
                        //println!("{:?} ### {:?}", network_packet.protocol, network_packet.arp.unwrap());
                    }
                    NetworkProtocol::IcmpIpv4 => {
                        println!("{:?} ### {:?} ### {:?}", network_packet.protocol, network_packet.ip.unwrap(), network_packet.icmp);
                    }
                    NetworkProtocol::Ipv4 => {
                        println!("{:?} ### {:?}", network_packet.protocol, network_packet.ip.unwrap());
                    }
                    NetworkProtocol::Ipv6 => {
                        println!("{:?} ### {:?}", network_packet.protocol, network_packet.ip.unwrap());
                    }
                    NetworkProtocol::Unknown => {
                        println!("Unimplemented network for {:?}", network_packet.payload)
                    }
                    NetworkProtocol::IcmpIpv6 => {
                        //println!("{:?} ### {:?} ### {:?}", network_packet.protocol, network_packet.ip.unwrap(), network_packet.icmp);
                    }
                }
            }
            _ => println!("Unimplemented data link for {:?}", data_link_packet.payload),
        }

        /*
        // Layer 3: network
        if let Some(ipv4) = Ipv4Packet::new(data_link_packet.payload()) {
            //println!("ipv4 pac! {:?}", ipv4);

            // Layer 4: transport
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                //println!("tcp v4 pac! {:?}", tcp);
            }
            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                if udp.get_destination() == 53 || udp.get_source() == 53 {
                    if let Some(dns) = DnsPacket::new(udp.payload()) {}
                }
            }
        }
        if let Some(ipv6) = Ipv6Packet::new(data_link_packet.payload()) {
            //println!("ipv6 pac! {:?}", ipv6);
            if let Some(tcp) = TcpPacket::new(ipv6.payload()) {
                //println!("tcp v6 pac! {:?}", tcp);
            }
            if let Some(udp) = UdpPacket::new(ipv6.payload()) {
                if udp.get_destination() == 53 || udp.get_source() == 53 {
                    if let Some(dns) = DnsPacket::new(udp.payload()) {}
                }
            }
        }
        if let Some(icmp) = IcmpPacket::new(data_link_packet.payload()) {
            //println!("icmp pac! {:?}", icmp);
        }

        /* if let Some(icmp) = IcmpPacket::new(data_link_packet.payload()) {
            println!("icmp pac! {:?}", icmp);
        }
        if let Some(tcp) = TcpPacket::new(data_link_packet.payload()) {
          //  println!("tcp src {:?}", tcp.get_source());
          //  println!("tcp dst {:?}", tcp.get_destination());
            //if let Some(dns) = DnsPacket::new(tcp.payload()) {
             //  println!("tcp pac! {:?}", dns);
            //}

        }
        if let Some(udp) = UdpPacket::new(data_link_packet.payload()) {
            println!("udp  {:?}", udp);
          //  println!("udp src {:?}", udp.get_source());
          //  println!("udp dst {:?}", udp.get_destination());
            if udp.get_destination() == 53 {
                if let Some(dns) = DnsPacket::new(udp.payload()) {
                  println!("dnsssss pac! {:?}", dns);
                }
            }
            //if let Some(dns) = DnsPacket::new(tcp.payload()) {
            //  println!("tcp pac! {:?}", dns);
            //}

        }
        //if let Some(dns) = DnsPacket::new(packet.data) {
        //    println!("tcp pac! {:?}", dns);
        //}*/*/
    }
}

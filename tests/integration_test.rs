use etherparse::{IpHeader, PacketBuilder, PacketHeaders, TransportHeader};
use serial_test::serial;
#[cfg(target_family = "unix")]
use std::io::ErrorKind;
use std::io::{IoSlice, Read, Write};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
#[cfg(not(target_os = "macos"))]
use utuntap::tap;
use utuntap::tun;

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn tun_sents_packets() {
    let mut file = tun::OpenOptions::new()
        .packet_info(false)
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    socket
        .send_to(&data, "10.10.10.2:4242")
        .expect("failed to send data");
    let mut buffer = [0; 50];
    let number = file.read(&mut buffer).expect("failed to receive data");
    assert_eq!(number, 38);
    let packet = &buffer[..number];
    if let PacketHeaders {
        ip: Some(IpHeader::Version4(ip_header, _)),
        transport: Some(TransportHeader::Udp(udp_header)),
        payload,
        ..
    } = PacketHeaders::from_ip_slice(&packet).expect("failed to parse packet")
    {
        assert_eq!(ip_header.source, [10, 10, 10, 1]);
        assert_eq!(ip_header.destination, [10, 10, 10, 2]);
        assert_eq!(udp_header.source_port, 2424);
        assert_eq!(udp_header.destination_port, 4242);
        assert_eq!(payload, data);
    } else {
        assert!(false, "incorrect packet");
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn tun_sents_packets_with_packet_info() {
    let mut file = tun::OpenOptions::new()
        .packet_info(true)
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    socket
        .send_to(&data, "10.10.10.2:4242")
        .expect("failed to send data");
    let mut buffer = [0; 50];
    let number = file.read(&mut buffer).expect("failed to receive data");
    assert_eq!(number, 42);
    assert_eq!(&buffer[..4], [0, 0, 8, 0]);
    let packet = &buffer[4..number];
    if let PacketHeaders {
        ip: Some(IpHeader::Version4(ip_header, _)),
        transport: Some(TransportHeader::Udp(udp_header)),
        payload,
        ..
    } = PacketHeaders::from_ip_slice(&packet).expect("failed to parse packet")
    {
        assert_eq!(ip_header.source, [10, 10, 10, 1]);
        assert_eq!(ip_header.destination, [10, 10, 10, 2]);
        assert_eq!(udp_header.source_port, 2424);
        assert_eq!(udp_header.destination_port, 4242);
        assert_eq!(payload, data);
    } else {
        assert!(false, "incorrect packet");
    }
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn tun_receives_packets() {
    let mut file = tun::OpenOptions::new()
        .packet_info(false)
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let packet = {
        let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
        builder
            .write(&mut packet, &data)
            .expect("failed to build packet");
        packet
    };
    file.write(&packet).expect("failed to send packet");
    let mut buffer = [0; 50];
    let (number, source) = socket
        .recv_from(&mut buffer)
        .expect("failed to receive packet");
    assert_eq!(number, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buffer[..number]);
}

#[cfg(target_os = "linux")]
#[test]
#[serial]
fn tun_receives_packets_with_packet_info() {
    let mut file = tun::OpenOptions::new()
        .packet_info(true)
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let packet_info = [0u8, 0, 8, 0];
    let packet = {
        let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
        builder
            .write(&mut packet, &data)
            .expect("failed to build packet");
        packet
    };
    let iovec = [IoSlice::new(&packet_info), IoSlice::new(&packet)];
    file.write_vectored(&iovec).expect("failed to send packet");
    let mut buffer = [0; 50];
    let (number, source) = socket
        .recv_from(&mut buffer)
        .expect("failed to receive packet");
    assert_eq!(number, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buffer[..number]);
}

#[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
#[test]
#[serial]
fn tun_sents_packets() {
    let mut file = tun::OpenOptions::new()
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    socket
        .send_to(&data, "10.10.10.2:4242")
        .expect("failed to send data");
    let mut buffer = [0; 50];
    let number = file.read(&mut buffer).expect("failed to receive data");
    assert_eq!(number, 42);
    assert_eq!(&buffer[..4], [0u8, 0, 0, 2]);
    let packet = &buffer[4..number];
    if let PacketHeaders {
        ip: Some(IpHeader::Version4(ip_header, _)),
        transport: Some(TransportHeader::Udp(udp_header)),
        payload,
        ..
    } = PacketHeaders::from_ip_slice(&packet).expect("failed to parse packet")
    {
        assert_eq!(ip_header.source, [10, 10, 10, 1]);
        assert_eq!(ip_header.destination, [10, 10, 10, 2]);
        assert_eq!(udp_header.source_port, 2424);
        assert_eq!(udp_header.destination_port, 4242);
        assert_eq!(payload, data);
    } else {
        assert!(false, "incorrect packet");
    }
}

#[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
#[test]
#[serial]
fn tun_receives_packets() {
    let mut file = tun::OpenOptions::new()
        .open(10)
        .expect("failed to open device");
    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let family = [0u8, 0, 0, 2];
    let packet = {
        let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
        builder
            .write(&mut packet, &data)
            .expect("failed to build packet");
        packet
    };
    let iovec = [IoSlice::new(&family), IoSlice::new(&packet)];
    file.write_vectored(&iovec).expect("failed to send packet");
    let mut buffer = [0; 50];
    let (number, source) = socket
        .recv_from(&mut buffer)
        .expect("failed to receive packet");
    assert_eq!(number, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buffer[..number]);
}

#[cfg(target_os = "macos")]
#[test]
#[serial]
fn tun_sents_packets() {
    let mut file = tun::OpenOptions::new()
        .open(10)
        .expect("failed to open device");

    std::process::Command::new("ifconfig")
        .arg("utun10")
        .arg("10.10.10.1")
        .arg("10.10.10.2")
        .arg("netmask")
        .arg("255.255.255.255")
        .status()
        .unwrap();

    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    socket
        .send_to(&data, "10.10.10.2:4242")
        .expect("failed to send data");
    let mut buffer = [0; 50];
    let number = file.read(&mut buffer).expect("failed to receive data");
    assert_eq!(number, 42);
    assert_eq!(&buffer[..4], [0u8, 0, 0, 2]);
    let packet = &buffer[4..number];
    if let PacketHeaders {
        ip: Some(IpHeader::Version4(ip_header, _)),
        transport: Some(TransportHeader::Udp(udp_header)),
        payload,
        ..
    } = PacketHeaders::from_ip_slice(&packet).expect("failed to parse packet")
    {
        assert_eq!(ip_header.source, [10, 10, 10, 1]);
        assert_eq!(ip_header.destination, [10, 10, 10, 2]);
        assert_eq!(udp_header.source_port, 2424);
        assert_eq!(udp_header.destination_port, 4242);
        assert_eq!(payload, data);
    } else {
        assert!(false, "incorrect packet");
    }
}

#[cfg(target_os = "macos")]
#[test]
#[serial]
fn tun_receives_packets() {
    let mut file = tun::OpenOptions::new()
        .open(10)
        .expect("failed to open device");

    std::process::Command::new("ifconfig")
        .arg("utun10")
        .arg("10.10.10.1")
        .arg("10.10.10.2")
        .arg("netmask")
        .arg("255.255.255.255")
        .status()
        .unwrap();

    let data = [1; 10];
    let socket = UdpSocket::bind("10.10.10.1:2424").expect("failed to bind to address");
    let builder = PacketBuilder::ipv4([10, 10, 10, 2], [10, 10, 10, 1], 20).udp(4242, 2424);
    let family = [0u8, 0, 0, 2];
    let packet = {
        let mut packet = Vec::<u8>::with_capacity(builder.size(data.len()));
        builder
            .write(&mut packet, &data)
            .expect("failed to build packet");
        packet
    };
    let iovec = [IoSlice::new(&family), IoSlice::new(&packet)];
    file.write_vectored(&iovec).expect("failed to send packet");
    let mut buffer = [0; 50];
    let (number, source) = socket
        .recv_from(&mut buffer)
        .expect("failed to receive packet");
    assert_eq!(number, 10);
    assert_eq!(source.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 10, 2)));
    assert_eq!(source.port(), 4242);
    assert_eq!(data, &buffer[..number]);
}

#[cfg(target_family = "unix")]
#[test]
#[serial]
fn tun_non_blocking_io() {
    let mut file = tun::OpenOptions::new()
        .nonblock(true)
        .open(11)
        .expect("failed to open device");
    let mut buffer = [0; 10];
    while file.read(&mut buffer).is_ok() {}
    let error = file.read(&mut buffer).err().unwrap();
    assert_eq!(error.kind(), ErrorKind::WouldBlock);
}

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
#[test]
#[serial]
fn tap_non_blocking_io() {
    let mut file = tap::OpenOptions::new()
        .nonblock(true)
        .open(11)
        .expect("failed to open device");
    let mut buffer = [0; 10];
    while file.read(&mut buffer).is_ok() {}
    let error = file.read(&mut buffer).err().unwrap();
    assert_eq!(error.kind(), ErrorKind::WouldBlock);
}

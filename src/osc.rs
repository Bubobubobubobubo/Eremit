use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::time::Duration;
use std::{env, f32, thread};

pub fn addr_from_string(addr: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(addr).unwrap()
}

pub fn create_socket(addr: &str) -> UdpSocket {
    let addr = addr_from_string(addr);
    let socket = UdpSocket::bind(addr).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket
}

pub fn send_message(socket: &UdpSocket, message: OscMessage, to_addr: &str) {
    let packet = OscPacket::Message(message);
    let encoded = encoder::encode(&packet).unwrap();
    socket.send_to(&encoded, addr_from_string(to_addr)).unwrap()
}
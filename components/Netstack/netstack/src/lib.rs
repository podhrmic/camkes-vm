#![feature(libc)]
#![feature(lang_items)]
extern crate libc;
extern crate smoltcp;

use std::collections::BTreeMap;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint};
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache};
use smoltcp::socket::SocketSet;
use smoltcp::socket::{UdpSocketBuffer, UdpSocket, UdpPacketMetadata};
use smoltcp::time::Instant;



use libc::{c_void, memcpy};
use std::{io, mem};

#[no_mangle]
extern "C" {
  fn printf(val: *const i8);
}


#[no_mangle]
pub extern "C" fn run() -> isize {
    unsafe{ printf(b"Hello from Rust, starting main\n\0".as_ptr() as *const i8); }
    main();
    unsafe{ printf(b"Main done\n\0".as_ptr() as *const i8); }
    0
}

extern "C" {
    fn ethdriver_mac(b1: *mut u8, b2: *mut u8, b3: *mut u8, b4: *mut u8, b5: *mut u8, b6: *mut u8);
}


/// Event callback I believe
/// `badge` is not used
#[no_mangle]
pub extern "C" fn ethdriver_has_data_callback(_badge: u32) {     unsafe{ printf(b"Has data callback!\n\0".as_ptr() as *const i8); } }


extern "C" {
    // to match C signatures
    static ethdriver_buf: *mut c_void;
    fn ethdriver_tx(len: i32) -> i32;
    fn ethdriver_rx(len: *mut i32) -> i32;
}

/// A backend for smoltcp, to be called from its `phy` module
/// Transmits a slice from the client application by copying data
/// into `ethdriver_buf` and consequently calling `ethdriver_tx()`
/// Returns either number of transmitted bytes or an error
fn sel4_eth_transmit(buf: &mut [u8]) -> i32 {
    unsafe {
        let local_buf_ptr = mem::transmute::<*mut u8, *mut c_void>(buf.as_mut_ptr());
        assert!(!ethdriver_buf.is_null());
        memcpy(ethdriver_buf, local_buf_ptr, buf.len());
        unsafe{ printf(format!("Ethdriver tx sending {} bytes\n\0",buf.len()).as_ptr() as *const i8); }          
        ethdriver_tx(buf.len() as i32)
    }
}

/// A backend for smoltcp, to be called from its `phy` module
/// Attempt to receive data from the ethernet driver
/// Call `ethdriver_rx` and cast the results.
/// Returns either a vector of received bytes, or an error
fn sel4_eth_receive() -> i32 {
    let mut len = 0;
    unsafe {
        let res = ethdriver_rx(&mut len);
        unsafe{ printf(format!("Ethdriver rx len={}\n\0",len).as_ptr() as *const i8); }  
        res   
/*
        assert!(!ethdriver_buf.is_null());
        // create a slice of length `len` from `ethdriver_buf`
        let local_buf_ptr = mem::transmute::<*mut c_void, *mut u8>(ethdriver_buf);
        let slice = slice::from_raw_parts(local_buf_ptr, len as usize);

        // instead of dealing with the borrow checker, copy slice in to a vector
        let mut vec = Vec::new();
        vec.extend_from_slice(slice);
        Ok(vec)
*/
    }
}



/// Pass the device MAC address to the callee
fn get_device_mac() -> EthernetAddress {
    let mut b1: u8 = 0;
    let mut b2: u8 = 0;
    let mut b3: u8 = 0;
    let mut b4: u8 = 0;
    let mut b5: u8 = 0;
    let mut b6: u8 = 0;

    unsafe {
        ethdriver_mac(&mut b1, &mut b2, &mut b3, &mut b4, &mut b5, &mut b6);
    }

    EthernetAddress([b1, b2, b3, b4, b5, b6])
}


fn main() {
    //unsafe{ printf(b"A\n\0".as_ptr() as *const i8); }
    //let device = Sel4Device::new();
    unsafe{ printf(b"B\n\0".as_ptr() as *const i8); }
    let neighbor_cache = NeighborCache::new(BTreeMap::new());
    unsafe{ printf(b"C\n\0".as_ptr() as *const i8); }
    let udp1_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 64]);
    let udp1_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 128]);
    let udp1_socket = UdpSocket::new(udp1_rx_buffer, udp1_tx_buffer);
    unsafe{ printf(b"D\n\0".as_ptr() as *const i8); }
    let udp2_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 64]);
    let udp2_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY], vec![0; 128]);
    let udp2_socket = UdpSocket::new(udp2_rx_buffer, udp2_tx_buffer);
    unsafe{ printf(b"E\n\0".as_ptr() as *const i8); }

    let ethernet_addr = get_device_mac();
    unsafe{ printf(format!("Ethaddr={}\n\0",ethernet_addr).as_ptr() as *const i8); }
/*
    let ip_addrs = [IpCidr::new(IpAddress::v4(192, 168, 179, 2), 24)];
    let mut iface = EthernetInterfaceBuilder::new(device)
        .ethernet_addr(ethernet_addr)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(ip_addrs)
        .finalize();
    unsafe{ printf(b"F\n\0".as_ptr() as *const i8); }
    let mut sockets = SocketSet::new(vec![]);
    let udp1_handle = sockets.add(udp1_socket);
    let udp2_handle = sockets.add(udp2_socket);
    unsafe{ printf(b"G\n\0".as_ptr() as *const i8); }
*/
    let mut ms: u64 = 1;

    loop {
      ms +=1;
    if (ms % 100000000) == 0 {
      unsafe{ printf(format!("Netstack: Poll time: {}\n\0",ms).as_ptr() as *const i8); }
      unsafe{ printf(format!("Netstack: Sending data\n\0").as_ptr() as *const i8); }
      // send ARP request
      let mut v = vec![ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // destination multicast
                        0x52, 0x54, 0x00, 0x12, 0x34, 0x56, // source MAC (my)
                        0x08, 0x06, // type ARP
                        0x00, 0x01, // hw type Ethernet
                        0x08, 0x00, // Ipv4
                        0x06, // hw size
                        0x04, // protocol size
                        0x00, 0x01, // opcode request
                        0x52, 0x54, 0x00, 0x12, 0x34, 0x56, // sender mac
                        0xc0, 0xa8, 0xb3, 0x32, // sender IP  192.168.179.50
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // target mac
                        0xc0, 0xa8, 0xb3, 0x01, // target IP 192.168.179.1
                        ];
      let res = sel4_eth_transmit(&mut v.as_mut_slice());
      unsafe{ printf(format!("Netstack: eth transmit result={}\n\0",res).as_ptr() as *const i8); }


      let res = sel4_eth_receive();
      unsafe{ printf(format!("Netstack: eth receive result={}\n\0",res).as_ptr() as *const i8); }
    }

    }

/*
    loop {
// we don't have system time:-(
//        let timestamp = Instant::now();
        ms +=1;

    if (ms % 100000) == 0 {
      unsafe{ printf(format!("Poll time: {}\n\0",ms).as_ptr() as *const i8); }
    }
    

        let timestamp = Instant::from_millis(ms);
        iface.poll(&mut sockets, timestamp).expect("poll error");

        // udp:6969: respond "hello"
        {
            let mut socket = sockets.get::<UdpSocket>(udp1_handle);
            if !socket.is_open() {
    unsafe{ printf(format!("socket is not open\n\0").as_ptr() as *const i8); }
                socket.bind(6969).unwrap()
            }

            if socket.can_send() {
    unsafe{ printf(format!("socket can send()\n\0").as_ptr() as *const i8); }
                let data = b"hello\n";
                let endpoint = IpEndpoint::new(IpAddress::v4(192,168,179,1), 6666);
                socket.send_slice(data, endpoint).unwrap();
            }


            let client = match socket.recv() {
                Ok((data, endpoint)) => {
    unsafe{ printf(format!("socket can recv(), endppoint={}\n\0",endpoint).as_ptr() as *const i8); }
                    Some(endpoint)
                }
                Err(_) => None,
            };
            if let Some(endpoint) = client {
                let data = b"hello\n";
                socket.send_slice(data, endpoint).unwrap();
            }
        }

        // udp:6942: echo with reverse
        {
            let mut socket = sockets.get::<UdpSocket>(udp2_handle);
            if !socket.is_open() {
                socket.bind(6942).unwrap()
            }

            let mut rx_data = Vec::new();
            let client = match socket.recv() {
                Ok((data, endpoint)) => {
                    rx_data.extend_from_slice(data);
                    Some(endpoint)
                }
                Err(_) => None,
            };

            if let Some(endpoint) = client {
                if rx_data.len() > 0 {
                    let mut data = rx_data.split(|&b| b == b'\n').collect::<Vec<_>>().concat();
                    data.reverse();
                    data.extend(b"\n");
                    socket.send_slice(&data, endpoint).unwrap();
                }
            }
        }
    }
*/
}

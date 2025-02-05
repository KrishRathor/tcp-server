#![allow(dead_code, unused_variables, unused_imports)]

use std::{ffi::CString, io::stdout};

use libc::{
    accept, bind, htons, listen, sockaddr, sockaddr_in, socket, AF_INET, IPPROTO_TCP, SOCK_STREAM,
};

fn main() {
    // step 1 => create: socket use socket syscall with AF_INET, SOCK_STREAM and IPPROTO_TCP
    // this returns a fd for socket and -1 for error;

    unsafe {
        let socket_fd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);

        if socket_fd == -1 {
            println!("Error creating socket");
        }

        println!("{}", socket_fd);

        let ip = "0.0.0.0";
        let port = 8080;

        let mut addr: sockaddr_in = std::mem::zeroed();
        addr.sin_family = AF_INET as u16;
        addr.sin_port = htons(port);
        addr.sin_addr = convert_ip_to_in_addr(ip);

        let bind_result = bind(
            socket_fd,
            &addr as *const sockaddr_in as *const libc::sockaddr,
            std::mem::size_of::<sockaddr_in>() as u32,
        );

        if bind_result == -1 {
            libc::perror(b"Error in binding\0".as_ptr() as *const i8);
            return;
        }

        println!("Binding Result: {}", bind_result);

        let listen_result = listen(socket_fd, 1);

        if listen_result == -1 {
            println!("Error in listen!");
            return;
        }

        println!("Server is listening for connections...");

        loop {
            let client_sock = accept(socket_fd, std::ptr::null_mut(), std::ptr::null_mut());

            if client_sock == -1 {
                println!("Error inc client sock");
                return;
            }

            println!("Client connected {}", client_sock);
            let mut buffer = [0u8; 1024];

            let bytes_received = libc::recv(
                client_sock,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0, // No special flags
            );

            if bytes_received > 0 {
                let received_data = String::from_utf8_lossy(&buffer[..bytes_received as usize]);
                println!("Received data: {}", received_data);
            } else {
                println!("Client disconnected or error receiving data");
            }
        }
    }
}

fn convert_ip_to_in_addr(ip: &str) -> libc::in_addr {
    let octets: Vec<u8> = ip
        .split('.')
        .map(|octet| octet.parse::<u8>().expect("Invalid octet"))
        .collect();

    libc::in_addr {
        s_addr: ((octets[0] as u32) << 24)
            | ((octets[1] as u32) << 16)
            | ((octets[2] as u32) << 8)
            | (octets[3] as u32),
    }
}

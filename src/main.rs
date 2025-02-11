#![allow(dead_code, unused_variables, unused_imports)]

use errno::errno;
use libc::{
    self, accept, bind, fcntl, fd_set, htons, listen, recv, select, sockaddr, sockaddr_in, socket,
    timeval, AF_INET, FD_ISSET, FD_SET, FD_ZERO, F_SETFL, IPPROTO_TCP, O_NONBLOCK, SOCK_STREAM,
};
use std::{ffi::CString, io::stdout};

const MAX_CLIENTS: usize = 1024;

fn main() {
    unsafe {
        let socket_fd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if socket_fd == -1 {
            println!("Error creating socket");
        }

        let flags = fcntl(socket_fd, libc::F_GETFL);
        if flags == -1 {
            println!("Error getting socket flags");
        } else {
            if fcntl(socket_fd, F_SETFL, flags | O_NONBLOCK) == -1 {
                println!("Erorr setting 0 _NONBLOCK")
            }
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

        let mut client_fds: Vec<i32> = vec![socket_fd];

        loop {
            let mut read_fds: fd_set = std::mem::zeroed();
            FD_ZERO(&mut read_fds);

            let mut max_fd = socket_fd;
            for &fd in &client_fds {
                FD_SET(fd, &mut read_fds);
                if fd > max_fd {
                    max_fd = fd;
                }
            }

            let timeout = timeval {
                tv_sec: 5,
                tv_usec: 0,
            };

            let ready_count = select(
                max_fd + 1,
                &mut read_fds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &timeout as *const timeval as *mut timeval,
            );

            if ready_count == -1 {
                println!("Error in select!");
                break;
            }

            if ready_count == 0 {
                println!("no clients yet");
            }

            if FD_ISSET(socket_fd, &read_fds) {
                let client_sock = accept(socket_fd, std::ptr::null_mut(), std::ptr::null_mut());
                if client_sock != -1 {
                    println!("Client connected {}", client_sock);
                    client_fds.push(client_sock);
                }
            }

            for &client in &client_fds {
                if client != socket_fd && FD_ISSET(client, &read_fds) {
                    let mut buffer = [0u8; 1024];
                    let bytes_received = recv(
                        client,
                        buffer.as_mut_ptr() as *mut libc::c_void,
                        buffer.len(),
                        0,
                    );

                    if bytes_received == -1 {
                        let errno_val = errno().0;
                        if errno_val == libc::EAGAIN || errno_val == libc::EWOULDBLOCK {
                            println!("No data available yet, will try again later");
                        } else {
                            println!("Error receiving data: {}", errno_val);
                        }
                    } else if bytes_received > 0 {
                        let received_data =
                            String::from_utf8_lossy(&buffer[..bytes_received as usize]);
                        println!("Received from {}: {}", client, received_data);
                    }
                }
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


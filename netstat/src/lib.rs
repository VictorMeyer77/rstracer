use crate::error::Error;
use chrono::Local;
use procfs::net::{TcpNetEntry, UdpNetEntry};
use procfs::process::{FDTarget, Stat};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
pub mod error;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum SocketType {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Socket {
    pub socket_type: SocketType,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub state: String,
    pub rx_queue: u32,
    pub tx_queue: u32,
    pub uid: u32,
    pub inode: u64,
    pub pid: Option<i32>,
    pub command: Option<String>,
    pub _created_at: i64,
}

impl From<(TcpNetEntry, Option<i32>, Option<String>)> for Socket {
    fn from(value: (TcpNetEntry, Option<i32>, Option<String>)) -> Self {
        Socket {
            socket_type: SocketType::Tcp,
            local_address: value.0.local_address,
            remote_address: value.0.remote_address,
            state: format!("{:?}", value.0.state),
            rx_queue: value.0.rx_queue,
            tx_queue: value.0.rx_queue,
            uid: value.0.uid,
            inode: value.0.inode,
            pid: value.1,
            command: value.2,
            _created_at: Local::now().timestamp_millis(),
        }
    }
}

impl From<(UdpNetEntry, Option<i32>, Option<String>)> for Socket {
    fn from(value: (UdpNetEntry, Option<i32>, Option<String>)) -> Self {
        Socket {
            socket_type: SocketType::Udp,
            local_address: value.0.local_address,
            remote_address: value.0.remote_address,
            state: format!("{:?}", value.0.state),
            rx_queue: value.0.rx_queue,
            tx_queue: value.0.rx_queue,
            uid: value.0.uid,
            inode: value.0.inode,
            pid: value.1,
            command: value.2,
            _created_at: Local::now().timestamp_millis(),
        }
    }
}

fn inode_with_process() -> Result<HashMap<u64, Stat>, Error> {
    let processes = procfs::process::all_processes()?;
    let mut inode_with_pid: HashMap<u64, Stat> = HashMap::new();
    for process in processes.flatten() {
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd in fds {
                if let FDTarget::Socket(inode) = fd?.target {
                    inode_with_pid.insert(inode, stat.clone());
                }
            }
        };
    }
    Ok(inode_with_pid)
}

fn read_tcp_socket(inode_with_process: &HashMap<u64, Stat>) -> Result<Vec<Socket>, Error> {
    let mut socket_buffer: Vec<Socket> = vec![];
    let tcp_entries = procfs::net::tcp()?.into_iter().chain(procfs::net::tcp6()?);
    for tcp_entry in tcp_entries {
        if let Some(stat) = inode_with_process.get(&tcp_entry.inode) {
            socket_buffer.push(Socket::from((
                tcp_entry,
                Some(stat.pid),
                Some(stat.comm.clone()),
            )));
        } else {
            socket_buffer.push(Socket::from((tcp_entry, None, None)));
        }
    }
    Ok(socket_buffer)
}

fn read_udp_socket(inode_with_process: &HashMap<u64, Stat>) -> Result<Vec<Socket>, Error> {
    let mut socket_buffer: Vec<Socket> = vec![];
    let udp_entries = procfs::net::udp()?.into_iter().chain(procfs::net::udp6()?);
    for udp_entry in udp_entries {
        if let Some(stat) = inode_with_process.get(&udp_entry.inode) {
            socket_buffer.push(Socket::from((
                udp_entry,
                Some(stat.pid),
                Some(stat.comm.clone()),
            )));
        } else {
            socket_buffer.push(Socket::from((udp_entry, None, None)));
        }
    }
    Ok(socket_buffer)
}

fn read_socket() -> Result<Vec<Socket>, Error> {
    let mut socket_buffer: Vec<Socket> = vec![];
    let inode_with_process = inode_with_process()?;
    socket_buffer.extend_from_slice(&read_tcp_socket(&inode_with_process)?);
    socket_buffer.extend_from_slice(&read_udp_socket(&inode_with_process)?);
    Ok(socket_buffer)
}

pub async fn producer(
    sender: Sender<Socket>,
    stop_flag: &Arc<AtomicBool>,
    frequency: u64,
) -> Result<(), Error> {
    while !stop_flag.load(Ordering::Relaxed) {
        let sockets = read_socket()?;
        for socket in sockets {
            if let Err(e) = sender.send(socket).await {
                return Err(Error::Channel(Box::new(e)));
            }
        }
        sleep(Duration::from_millis(frequency)).await
    }

    Ok(())
}

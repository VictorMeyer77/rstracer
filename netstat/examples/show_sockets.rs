use netstat::{sockets, Socket};

fn main() {
    display(sockets().unwrap());
}

fn display(sockets: Vec<Socket>) {
    println!(
        "{0: <6} | {1: <5} | {2: <20} | {3: <20} | {4: <20} | {5: <8} | {6: <8} | {7: <8} | {8: <5}",
        "pid", "uid", "command", "local_address", "remote_address", "state", "rx_queue", "tx_queue", "inode"
    );
    sockets.iter().for_each(|socket| {
        println!(
            "{0: <6} | {1: <5} | {2: <20} | {3: <20} | {4: <20} | {5: <8} | {6: <8} | {7: <8} | {8: <5}",
            socket.pid.unwrap_or(-1),
            socket.uid,
            socket.command.clone().unwrap_or_else(|| "-".to_string()),
            socket.local_address,
            socket.remote_address,
            socket.state,
            socket.rx_queue,
            socket.tx_queue,
            socket.inode
        );
    });
}

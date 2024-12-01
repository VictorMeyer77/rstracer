use lsof::lsof::{lsof, FileType, OpenFile};

fn main() {
    display(lsof(&FileType::ALL).unwrap());
}

fn display(files: Vec<OpenFile>) {
    println!(
        "{0: <6} | {1: <5} | {2: <5} | {3: <10} | {4: <5} | {5: <10} | {6: <6} | {7: <5} | {8: <5}",
        "pid", "uid", "command", "fd", "type", "device", "size", "node", "name"
    );
    files.iter().for_each(|file| {
        let truncate_name = if file.name.len() < 100 {
            &file.name
        } else {
            &file.name[..100]
        };
        println!(
            "{0: <6} | {1: <5} | {2: <5} | {3: <10} | {4: <5} | {5: <10} | {6: <6} | {7: <5} | {8: <5}",
            file.pid,
            file.uid,
            file.command,
            file.fd,
            file._type,
            file.device,
            file.size,
            file.node,
            truncate_name
        );
    });
}

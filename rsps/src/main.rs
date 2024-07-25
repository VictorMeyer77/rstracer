use rsps::ps::{rsps, Process};

pub fn main() {
    display(rsps().unwrap())
}

fn display(processes: Vec<Process>) {
    println!(
        "{0: <6} | {1: <5} | {2: <5} | {3: <10} | {4: <5} | {5: <5} | {6: <6} | {7: <5}",
        "pid", "ppid", "uid", "lstart", "pcpu", "pmem", "status", "command"
    );
    processes.iter().for_each(|process| {
        let truncate_command = if process.command.len() < 100 {
            &process.command
        } else {
            &process.command[..100]
        };
        println!(
            "{0: <6} | {1: <5} | {2: <5} | {3: <10} | {4: <5} | {5: <5} | {6: <6} | {7: <5}",
            process.pid,
            process.ppid,
            process.uid,
            process.lstart,
            process.pcpu,
            process.pmem,
            process.status,
            truncate_command
        );
    });
}

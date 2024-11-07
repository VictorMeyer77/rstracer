use etc::etc::user::User;
use etc::etc::EtcReader;

fn main() {
    display(User::read_etc_file(None).unwrap());
}

fn display(users: Vec<User>) {
    println!("{0: <25} | {1: <25}", "name", "uid");
    users.iter().for_each(|user| {
        println!("{0: <25} | {1: <25}", user.name, user.uid);
    });
}

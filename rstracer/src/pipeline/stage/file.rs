use crate::pipeline::error::Error;
use etc::etc::host::Host;
use etc::etc::passwd::User;
use etc::etc::service::Service;
use etc::etc::EtcReader;

fn insert_service_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE gold_file_service;
    INSERT INTO gold_file_service (name, port, protocol, inserted_at) VALUES "#
        .to_string();

    let insert_values: Vec<String> = Service::read_etc_file(None)?
        .iter()
        .map(|service| {
            format!(
                "('{}', {}, '{}', CURRENT_TIMESTAMP)",
                service.name, service.port, service.protocol
            )
        })
        .collect();

    Ok(format!(
        "{} {}; COMMIT;",
        request_buffer,
        insert_values.join(",")
    ))
}

fn insert_host_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE gold_file_host;
    INSERT INTO gold_file_host (name, address, inserted_at) VALUES "#
        .to_string();

    let insert_values: Vec<String> = Host::read_etc_file(None)?
        .iter()
        .map(|host| format!("('{}', '{}', CURRENT_TIMESTAMP)", host.name, host.address))
        .collect();

    Ok(format!(
        "{} {}; COMMIT;",
        request_buffer,
        insert_values.join(",")
    ))
}

fn insert_user_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE gold_file_user;
    INSERT INTO gold_file_user (name, uid, inserted_at) VALUES "#
        .to_string();

    let insert_values: Vec<String> = User::read_etc_file(None)?
        .iter()
        .map(|user| format!("('{}', '{}', CURRENT_TIMESTAMP)", user.name, user.uid))
        .collect();

    Ok(format!(
        "{} {}; COMMIT;",
        request_buffer,
        insert_values.join(",")
    ))
}

pub fn request() -> Result<String, Error> {
    Ok(format!(
        "{} {} {}",
        insert_service_request()?,
        insert_host_request()?,
        insert_user_request()?
    ))
}

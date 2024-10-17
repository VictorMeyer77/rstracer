use crate::pipeline::error::Error;
use etc::etc::host::Host;
use etc::etc::service::Service;
use etc::etc::EtcReader;

fn insert_services_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE memory.gold_dim_services;
    INSERT INTO memory.gold_dim_services (name, port, protocol, inserted_at) VALUES "#
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

fn insert_hosts_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE memory.gold_dim_hosts;
    INSERT INTO memory.gold_dim_hosts (name, address, inserted_at) VALUES "#
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

pub fn request() -> Result<String, Error> {
    Ok(format!(
        "{} {}",
        insert_services_request()?,
        insert_hosts_request()?
    ))
}

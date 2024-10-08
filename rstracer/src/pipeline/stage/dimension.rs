use crate::pipeline::error::Error;
use etc::etc::service::Service;
use etc::etc::EtcReader;

fn insert_services_request() -> Result<String, Error> {
    let request_buffer = r#"BEGIN; TRUNCATE memory.gold_dim_services;
    INSERT INTO memory.gold_dim_services (name, port, protocol, updated_at) VALUES "#
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

pub fn request() -> Result<String, Error> {
    insert_services_request()
}

use lsof::lsof::OpenFile;
use network::capture::Capture;
use ps::ps::Process;
use uuid::Uuid;

pub trait Bronze {
    fn to_sql(&self) -> String;
}

pub trait BronzeBatch {
    fn get_insert_header() -> String;

    fn to_insert_value(&self) -> String;
}

impl BronzeBatch for Process {
    fn get_insert_header() -> String {
        r#"INSERT INTO memory.bronze_process_list
        (pid, ppid, uid, lstart, pcpu, pmem, status, command, created_at, inserted_at, brz_ingestion_duration)
        VALUES "#
            .to_string()
    }

    fn to_insert_value(&self) -> String {
        format!("({}, {}, {}, TO_TIMESTAMP({}), {}, {}, '{}', '{}', TO_TIMESTAMP({8}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({8})::TIMESTAMP))",
            self.pid,
            self.ppid,
            self.uid,
            self.lstart,
            self.pcpu,
            self.pmem,
            self.status,
            self.command.replace('\'', "\""),
            self.created_at
        )
    }
}

impl BronzeBatch for OpenFile {
    fn get_insert_header() -> String {
        r#"INSERT INTO memory.bronze_open_files
        (command, pid, uid, fd, type, device, size, node, name, created_at, inserted_at, brz_ingestion_duration)
        VALUES "#
            .to_string()
    }

    fn to_insert_value(&self) -> String {
        format!(
            r#"('{}', {}, {}, '{}', '{}', '{}', {}, '{}', '{}', TO_TIMESTAMP({9}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({9})::TIMESTAMP))"#,
            self.command.replace('\'', "\""),
            self.pid,
            self.uid,
            self.fd,
            self._type,
            self.device,
            self.size,
            self.node,
            self.name.replace('\'', "\""),
            self.created_at
        )
    }
}

impl Bronze for Capture {
    fn to_sql(&self) -> String {
        let row_id = Uuid::new_v4().as_u64_pair().0;
        let mut request_buffer = "BEGIN; ".to_string();
        request_buffer.push_str(&format!(
            r#"INSERT OR REPLACE INTO memory.bronze_network_packet
        (_id, interface, length, created_at, inserted_at, brz_ingestion_duration) VALUES
        ({}, '{}', {}, TO_TIMESTAMP({3}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({3})::TIMESTAMP));"#,
            row_id,
            self.device.name,
            self.packet.len(),
            self.created_at
        ));
        request_buffer.push_str("COMMIT;");
        request_buffer
    }
}

use ps::ps::Process;

pub trait Bronze {
    fn to_sql(&self) -> String;
}

impl Bronze for Process {
    fn to_sql(&self) -> String {
        format!(
            r#"INSERT INTO memory.bronze_process_list
        (pid, ppid, uid, lstart, pcpu, pmem, status, command, created_at, inserted_at)
        VALUES ({}, {}, {}, to_timestamp({}), {}, {}, '{}', '{}', to_timestamp({}), current_timestamp);"#,
            self.pid,
            self.ppid,
            self.uid,
            self.lstart,
            self.pcpu,
            self.pmem,
            self.status,
            self.command,
            self.created_at
        )
    }
}

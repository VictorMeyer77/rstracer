use crate::data::{BronzeCompute, BronzeStorage, error};
use chrono::{Local, NaiveDateTime, Utc};
use std::process::Command;
use std::thread;
use polars::frame::DataFrame;
use polars::prelude::{NamedFrom, ParquetWriter, Series};

const PS_SEPARATOR: &str = "%@%";
const PS_ARGS: [&str; 64] = [
    "-e",
    "-o",
    "pid",
    "-o",
    "%%@%%",
    "-o",
    "ppid",
    "-o",
    "%%@%%",
    "-o",
    "user",
    "-o",
    "%%@%%",
    "-o",
    "uid",
    "-o",
    "%%@%%",
    "-o",
    "lstart",
    "-o",
    "%%@%%",
    "-o",
    "time",
    "-o",
    "%%@%%",
    "-o",
    "vsz",
    "-o",
    "%%@%%",
    "-o",
    "rss",
    "-o",
    "%%@%%",
    "-o",
    "pcpu",
    "-o",
    "%%@%%",
    "-o",
    "pmem",
    "-o",
    "%%@%%",
    "-o",
    "tty",
    "-o",
    "%%@%%",
    "-o",
    "stat",
    "-o",
    "%%@%%",
    "-o",
    "nlwp",
    "-o",
    "%%@%%",
    "-o",
    "pri",
    "-o",
    "%%@%%",
    "-o",
    "nice",
    "-o",
    "%%@%%",
    "-o",
    "args:10000",
    "--no-headers",
];

#[derive(Debug)]
struct Process {
    pid: Vec<u32>,     // Process ID
    ppid: Vec<u32>,    // Parent Process ID
    user: Vec<String>, // Username of the process owner
    uid: Vec<u32>,     // User ID of the process owner
    lstart: Vec<i64>,  // Exact date and time when the process started
    time: Vec<String>, // Total CPU time used by the process
    vsz: Vec<u64>,     // Virtual memory size (in KB)
    rss: Vec<u64>,     // Resident set size (in KB)
    pcpu: Vec<f32>,    // CPU usage percentage
    pmem: Vec<f32>,    // Memory usage percentage
    tty: Vec<String>,  // Terminal associated with the process
    stat: Vec<String>, // Process status
    nlwp: Vec<u32>,    // Number of threads in the process
    pri: Vec<i32>,     // Priority of the process
    nice: Vec<String>, // Nice value of the process
    args: Vec<String>, // Command with all its arguments
}

impl Process {

    pub fn new() -> Process {
        Process {
            pid: vec![],
            ppid: vec![],
            user: vec![],
            uid: vec![],
            lstart: vec![],
            time: vec![],
            vsz: vec![],
            rss: vec![],
            pcpu: vec![],
            pmem: vec![],
            tty: vec![],
            stat: vec![],
            nlwp: vec![],
            pri: vec![],
            nice: vec![],
            args: vec![],
        }
    }

    pub fn insert(&mut self, row: &str, utc_sec: i32) -> Result<(), error::Error> {
        let fields: Vec<&str> = row.split(PS_SEPARATOR).map(|field| field.trim()).collect();
        self.pid.push(fields[0].parse()?);
        self.ppid.push(fields[1].parse()?);
        self.user.push(fields[2].to_string());
        self.uid.push(fields[3].parse()?);
        self.lstart.push(Self::parse_lstart(fields[4])? - utc_sec as i64);
        self.time.push(fields[5].to_string());
        self.vsz.push(fields[6].parse()?);
        self.rss.push(fields[7].parse()?);
        self.pcpu.push(fields[8].parse()?);
        self.pmem.push(fields[9].parse()?);
        self.tty.push(fields[10].to_string());
        self.stat.push(fields[11].to_string());
        self.nlwp.push(fields[12].parse()?);
        self.pri.push(fields[13].parse()?);
        self.nice.push(fields[14].to_string());
        self.args.push(fields[15..].join(" "));
        Ok(())
    }

    fn parse_lstart(date_str: &str) -> Result<i64, error::Error> {
        let format = "%a %b %d %H:%M:%S %Y";
        Ok(NaiveDateTime::parse_from_str(date_str, format)?
            .and_utc()
            .timestamp())
    }

    fn to_polars_df(&self) -> DataFrame {
        DataFrame::new(vec![
            Series::new("pid", self.pid.clone()),
            Series::new("ppid", self.ppid.clone()),
            Series::new("user", self.user.clone()),
            Series::new("uid", self.uid.clone()),
            Series::new("lstart", self.lstart.clone()),
            Series::new("time", self.time.clone()),
            Series::new("vsz", self.vsz.clone()),
            Series::new("rss", self.rss.clone()),
            Series::new("pcpu", self.pcpu.clone()),
            Series::new("pmem", self.pmem.clone()),
            Series::new("tty", self.tty.clone()),
            Series::new("stat", self.stat.clone()),
            Series::new("nlwp", self.nlwp.clone()),
            Series::new("pri", self.pri.clone()),
            Series::new("nice", self.nice.clone()),
            Series::new("args", self.args.clone()),
        ]).unwrap()
    }

    fn vacuum(&mut self) {
        self.pid.clear();
        self.ppid.clear();
        self.user.clear();
        self.uid.clear();
        self.lstart.clear();
        self.time.clear();
        self.vsz.clear();
        self.rss.clear();
        self.pcpu.clear();
        self.pmem.clear();
        self.tty.clear();
        self.stat.clear();
        self.nlwp.clear();
        self.pri.clear();
        self.nice.clear();
        self.args.clear();
    }

    fn ps(&mut self) -> Result<(), error::Error> {
        let offset_in_sec = Local::now().offset().local_minus_utc();
        let ps = Command::new("ps").args(PS_ARGS).output()?;
        if ps.status.success() {
            String::from_utf8_lossy(&ps.stdout)
                .lines()
                .for_each(|line| self.insert(line, offset_in_sec).unwrap());// TODO gerer les mauvaises lignes
            Ok(())
        } else {
            panic!("Invalid ps syntax: {}", String::from_utf8_lossy(&ps.stderr));
        }
    }
}

impl BronzeCompute for Process {
    fn extract(&mut self) -> DataFrame {
        self.ps().unwrap();
        let df = self.to_polars_df();
        self.vacuum();
        df
    }

    fn transform(df: &mut DataFrame) -> () {
        let now = Utc::now().timestamp();
        df.with_column(Series::new("ctime", vec![now; df.height()])).unwrap();
    }

    fn load(df: DataFrame, storage: &mut BronzeStorage) -> () {
        /*if let Some(process_df) = &storage.process_df {
            storage.process_df = Some(process_df.vstack(&df).unwrap());
        } else {
            storage.process_df = Some(df);
        }*/
        let mut file = std::fs::File::create("bronze.parquet").unwrap();
        ParquetWriter::new(&mut file).finish(&mut df.clone()).unwrap();
    }
}


pub fn run() {
    let mut process = Process::new();
    let mut bronze_storage = BronzeStorage::new();
    let thread = thread::spawn(move || { process.launch(&mut bronze_storage); });

    thread.join().unwrap()

}

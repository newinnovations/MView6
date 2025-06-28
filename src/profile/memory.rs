use std::fs::File;
use std::io::{self, Read};

#[allow(dead_code)]
#[derive(Debug)]
pub struct MemoryUsage {
    total_program_size: usize,
    resident_set_size: usize,
    shared_pages: usize,
    text: usize,
    library: usize,
    data: usize,
    dt: usize,
}

fn read_memory_usage() -> io::Result<String> {
    let mut file = File::open("/proc/self/statm")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn get_memory_usage() -> Result<MemoryUsage, &'static str> {
    if let Ok(data) = read_memory_usage() {
        let fields: Vec<&str> = data.split_whitespace().collect();
        if fields.len() != 7 {
            return Err("Unexpected number of fields in /proc/self/statm");
        }

        let total_program_size = fields[0]
            .parse::<usize>()
            .map_err(|_| "Failed to parse total program size")?;
        let resident_set_size = fields[1]
            .parse::<usize>()
            .map_err(|_| "Failed to parse resident set size")?;
        let shared_pages = fields[2]
            .parse::<usize>()
            .map_err(|_| "Failed to parse shared pages")?;
        let text = fields[3]
            .parse::<usize>()
            .map_err(|_| "Failed to parse text")?;
        let library = fields[4]
            .parse::<usize>()
            .map_err(|_| "Failed to parse library")?;
        let data = fields[5]
            .parse::<usize>()
            .map_err(|_| "Failed to parse data")?;
        let dt = fields[6]
            .parse::<usize>()
            .map_err(|_| "Failed to parse dt")?;

        Ok(MemoryUsage {
            total_program_size,
            resident_set_size,
            shared_pages,
            text,
            library,
            data,
            dt,
        })
    } else {
        Err("Failed to read /proc/self/statm")
    }
}

#[allow(dead_code)]
pub fn dump_memory_usage() {
    match get_memory_usage() {
        Ok(usage) => {
            dbg!(usage);
        }
        Err(error) => {
            println!("{error}");
        }
    }
}

pub fn memory_short() -> String {
    match get_memory_usage() {
        Ok(usage) => {
            format!("(rss={0}, data={1})", usage.resident_set_size, usage.data)
        }
        Err(_) => String::default(),
    }
}

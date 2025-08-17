use std::fs::{create_dir, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use dirs_next::config_dir;

use crate::entry::Entry;

pub fn write_to_archive(entrys: &[Entry]) -> Result<(), std::io::Error> {
    let path = config_dir();

    let mut path = if let Some(path) = path {
        path
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Config dir not found",
        ));
    };

    path.push("hours");
    path.push("archive.csv");

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    for entry in entrys {
        writeln!(
            file,
            "{},{},{},{},{}",
            entry.date, entry.description, entry.start, entry.end, entry.hours
        )?;
    }

    writeln!(file, "-",)?;

    Ok(())
}

pub fn read_archive(entrys: &mut Vec<Entry>, total_hours: &mut f64) -> Result<(), std::io::Error> {
    let mut path = if let Some(path) = config_dir() {
        path
    } else {
        return Ok(());
    };

    path.push("hours");

    if !path.exists() {
        create_dir(&path)?;
    }

    path.push("archive.csv");

    if path.exists() {
        let file = File::open(path)?;
        let file = BufReader::new(file);

        for line in file.lines().flatten() {
            if line == "-" {
                // Empty entry for a break
                entrys.push(Entry::new(
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    0.0,
                ));
            } else if let Some(entry) = get_entry(line) {
                *total_hours += entry.hours;
                entrys.push(entry);
            }
        }
    }

    Ok(())
}

pub fn write_entry(entry: &Entry) {
    let path = config_dir();

    if let Some(mut path) = path {
        path.push("hours");
        path.push("entrys.csv");

        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path);

        if let Ok(mut file) = file {
            if let Err(e) = writeln!(
                file,
                "{},{},{},{},{}",
                entry.date, entry.description, entry.start, entry.end, entry.hours
            ) {
                eprintln!("Couldn't write to file: {e}");
            }
        }
    }
}

pub fn wipe_entrys() {
    let path = config_dir();

    if let Some(mut path) = path {
        path.push("hours");
        path.push("entrys.csv");

        let _file = OpenOptions::new().truncate(true).open(path);
    }
}

pub fn get_entry(line: String) -> Option<Entry> {
    let mut entry = line.splitn(5, ',');

    Some(Entry::new(
        entry.next()?.to_string(),
        entry.next()?.to_string(),
        entry.next()?.to_string(),
        entry.next()?.to_string(),
        entry.next()?.parse::<f64>().ok()?,
    ))
}

pub fn read_entrys(entrys: &mut Vec<Entry>, total_hours: &mut f64) -> Result<(), std::io::Error> {
    let mut path = if let Some(path) = config_dir() {
        path
    } else {
        return Ok(());
    };

    path.push("hours");

    if !path.exists() {
        create_dir(&path)?;
    }

    path.push("entrys.csv");

    if path.exists() {
        let file = File::open(path)?;
        let file = BufReader::new(file);

        for line in file.lines().flatten() {
            if let Some(entry) = get_entry(line) {
                *total_hours += entry.hours;
                entrys.push(entry);
            }
        }

        entrys.sort();
    }

    Ok(())
}

pub fn parse_difference(start: &str, end: &str) -> Option<f64> {
    let start = parse_time(start);
    let end = parse_time(end);

    if let (Some(start), Some(end)) = (start, end) {
        let mut difference = end - start;

        if difference < 0.0 {
            difference += 24.0;
        }

        return Some(difference);
    }

    None
}

fn parse_time(time: &str) -> Option<f64> {
    let bytes = time.bytes();

    let mut acc = 0.0;
    let mut hours = 0.0;

    for b in bytes {
        if b == b':' {
            hours = acc;
            acc = 0.0;
            continue;
        }

        if !b.is_ascii_digit() {
            break;
        }

        acc *= 10.0;
        acc += (b - b'0') as f64;
    }

    // There is no such thing as hour 0 so if hours is zeros its cause there was no :
    if hours == 0.0 {
        hours = acc;
        acc = 0.0;
    }

    if !(1.0..=12.0).contains(&hours) {
        return None;
    }

    if !(0.0..=59.0).contains(&acc) {
        return None;
    }

    if time.ends_with("pm") && hours.trunc() != 12.0 {
        hours += 12.0;
    }

    let hours = hours + (((acc / 60.0) * 100.0).round() / 100.0);

    if hours == 0.0 {
        return None;
    }

    Some(hours)
}

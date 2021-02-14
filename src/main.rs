use chrono::{DateTime, Local, TimeZone, Duration};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

extern crate clap;
use clap::{App, AppSettings, Arg};

fn main() {
    let app = App::new("UpdateFiles")
        .version("1.0")
        .author("y-i")
        .about("Find updated files")
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(Arg::from_usage("-d --duration [DURATION] 'duration sec'"));
    let matches = app.get_matches();

    let mut map: HashMap<DateTime<Local>, Vec<String>> = HashMap::new();

    let duration: i64 = if let Some(v) = matches.value_of("duration") {
        v.parse().unwrap()
    } else {
        60 * 60 * 24 * 365
    };

    let threshold_date = Local::now() - Duration::seconds(duration);

    match read_dir(&mut map, "./", threshold_date) {
        Ok(_) => print(map),
        Err(e) => eprintln!("{}", e),
    }
}

fn print(map: HashMap<DateTime<Local>, Vec<String>>) {
    let mut sorted: Vec<_> = map.iter().collect();
    sorted.sort_by_key(|a| a.0);

    for (key, values) in sorted.iter() {
        for value in values.iter() {
            println!("{} {}", key, value);
        }
    }
}

fn read_dir<P: AsRef<Path>>(map: &mut HashMap<DateTime<Local>, Vec<String>>, path: P, threshold_date: DateTime<Local>) -> io::Result<String> {
    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            // 再帰
            if let Err(e) = read_dir(map, entry.path(), threshold_date) {
                eprintln!("{}", e);
            }
        } else {
            let modified = metadata.modified();
            if modified.is_err() {
                continue;
            }

            let duration = modified
                .unwrap()
                .duration_since(std::time::SystemTime::UNIX_EPOCH);
            if duration.is_err() {
                continue;
            }

            let datetime: DateTime<Local> = Local.timestamp(duration.unwrap().as_secs() as i64, 0);

            if datetime < threshold_date {
                continue;
            }

            if let Some(files) = map.get_mut(&datetime) {
                // 既に時刻の配列があったらそこに追加する
                files.push(entry.path().display().to_string());
            } else {
                // まだ時刻の配列がなかったら配列を作ってそれを追加する
                let filename = vec![entry.path().display().to_string()];
                map.insert(datetime, filename);
            }
        }
    }

    Ok(String::from("OK"))
}

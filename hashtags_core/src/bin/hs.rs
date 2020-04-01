extern crate base64;
extern crate clap;
extern crate dirs;
use clap::{App, Arg};
use hashtags::core::HashTags;
use serde_json;
use std::string::String;

fn get_db_path() -> String {
    let mut home_path = dirs::home_dir().unwrap();
    home_path.push(".notes.db");
    String::from(home_path.to_str().unwrap())
}

fn main() {
    let matches = App::new("Hashtags App")
        .subcommand(
            App::new("create").about("create a new note").arg(
                Arg::with_name("note")
                    .short("n")
                    .takes_value(true)
                    .value_name("note"),
            ),
        )
        .subcommand(
            App::new("query")
                .about("query notes")
                .arg(
                    Arg::with_name("method")
                        .short("m")
                        .takes_value(true)
                        .possible_values(&["simple"]),
                )
                .arg(Arg::with_name("filter_string").short("f").takes_value(true))
                .arg(
                    Arg::with_name("output_format")
                        .short("o")
                        .takes_value(true)
                        .possible_values(&["simple", "json"])
                        .default_value("simple"),
                ),
        )
        .get_matches();

    let mut hs = HashTags::new(get_db_path().as_str()).unwrap();
    if let Some(m) = matches.subcommand_matches("create") {
        let note = m.value_of("note").unwrap();
        hs.create(note).unwrap();
        return;
    }
    if let Some(m) = matches.subcommand_matches("query") {
        let method = m.value_of("method").unwrap();
        let filter = m.value_of("filter_string").unwrap();
        let output = m.value_of("output_format").unwrap();
        let notes = match hs.query(method, filter) {
            Ok(n) => n,
            Err(e) => panic!(format!(
                "unable to query with ({}, {}), error: {}",
                method, filter, e
            )),
        };
        match output {
            "json" => {
                let s = match serde_json::to_string(&notes) {
                    Ok(s) => s,
                    Err(e) => panic!(format!("unable to serialize with JSON: {}", e)),
                };
                println!("{}", s);
            }
            "simple" => {
                for n in notes {
                    println!("{}", n.content);
                    println!("-----------------------------------------------------------");
                    println!("{}, Hash: {}", n.time_created, base64::encode(n.hash));
                    println!("===");
                }
            }
            _ => panic!("unknown output format: {}", output),
        };
        return;
    }
    panic!("no subcommand provided");
}

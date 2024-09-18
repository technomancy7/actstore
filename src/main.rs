use std::env;
use subprocess::Exec;
use subprocess::Redirection;
use colored::Colorize;
use sqlite;
use sqlite::State;
use dirs;

fn get_output(text: String) -> String {
    return Exec::shell(text).stdout(Redirection::Pipe)
      .capture().unwrap().stdout_str().trim_end().to_string();
}

fn print_help() {
    println!("ACTSTORE: v{} by Technomancer", env!("CARGO_PKG_VERSION"));
    println!("{} <key> <value>: sets a value", String::from("set").blue());
    println!("{} <key>: prints a value", String::from("get").blue());
    println!("{} <key>: deletes a value", String::from("delete").blue());
    println!("{}: show all entries", String::from("ls").blue());
    println!("{} <key> <note>: sets a note for the key", String::from("note").blue());
    println!("{}: runs value through xdg-open", String::from("open").blue());
    println!("{}: runs value as shell command", String::from("run").blue());
    println!("{}: treats value as path to text file to open in ACT_EDITOR", String::from("edit").blue());
    println!("{}: this help message", String::from("help").blue());
    println!("{}: print only the version", String::from("version").blue());
}

fn split_first_word(s: &str) -> (&str, &str) {
    match s.split_once(char::is_whitespace) {
        Some((first, rest)) => (first, rest.trim_start()),
        None => (s, ""),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut path = dirs::home_dir().ok_or("Could not find home directory").expect("Err");
    path.push("actstore.db");
    let connection = sqlite::open(path.clone()).unwrap();

    match args.len() {
        1 => print_help(),
        2 => {
            match args[1].as_str() {
                "help" | "h" => print_help(),
                "version" | "ver" | "v" => println!(env!("CARGO_PKG_VERSION")),
                "ls" => {
                    let query = format!("SELECT * FROM astore;");
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let key = statement.read::<String, _>("key").unwrap().blue();
                        let val = statement.read::<String, _>("value").unwrap().green();
                        let note = statement.read::<String, _>("note").unwrap().yellow();
                        if !note.is_empty() {
                            println!("{}: {}  # {}", key, val, note)
                        } else {
                            println!("{}: {}", key, val)
                        }
                    }
                },
                _ => {
                    eprintln!("error: invalid command");
                    print_help();
                },
            }
        },
        3.. => {
            let cmd = &args[1];
            let ln = &args[2..].join(" ");

            match &cmd[..] {
                "d" | "del" | "delete" | "unset" | "rm" | "remove" | "rem" => {
                    let mut exists = false;

                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        exists = true;
                    }

                    if exists == true {
                        connection.execute(format!("DELETE FROM astore WHERE key = '{}';", ln)).unwrap();
                        println!("Dropping key {}", ln);
                    } else {
                        println!("Key {} not found.", ln);
                    }

                },
                "note" => {
                    let (key, value) = split_first_word(ln);

                    //Sanity check, see if it exists already
                    let query = format!("SELECT * FROM astore where key = '{}';", key);
                    let mut statement = connection.prepare(query).unwrap();
                    let mut found = false;
                    while let Ok(State::Row) = statement.next() {
                        connection.execute(format!("UPDATE astore SET note = '{}' WHERE key = '{}'", value, key)).unwrap();
                        println!("Updating note...");
                        found = true;
                    }
                    if !found {
                        println!("Entry not found")
                    }
                },
                "set" | "save" | "sv" | "s" => {
                    let (key, value) = split_first_word(ln);

                    //Sanity check, see if it exists already
                    let query = format!("SELECT * FROM astore where key = '{}';", key);
                    let mut statement = connection.prepare(query).unwrap();
                    let mut found = false;
                    while let Ok(State::Row) = statement.next() {
                        connection.execute(format!("UPDATE astore SET value = '{}' WHERE key = '{}'", value, key)).unwrap();
                        println!("Updating existing entry...");
                        found = true;
                    }

                    if !found {
                        let query = format!("
                            CREATE TABLE IF NOT EXISTS astore (key TEXT, value TEXT, note TEXT);
                            INSERT INTO astore VALUES ('{}', '{}', '{}');
                        ", key, value, String::new());
                        connection.execute(query).unwrap();
                        println!("Saved.");
                    }

                },
                "get" | "show" | "g" => {
                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let key = statement.read::<String, _>("key").unwrap().blue();
                        let val = statement.read::<String, _>("value").unwrap().green();
                        let note = statement.read::<String, _>("note").unwrap().blue();
                        if !note.is_empty() {
                            println!("{}: {}  # {}", key, val, note)
                        } else {
                            println!("{}: {}", key, val)
                        }
                    }
                },
                "open" | "url" => {
                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let val = statement.read::<String, _>("value").unwrap();
                        get_output(format!("xdg-open {}", val));
                    }
                },
                "run" => {
                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let val = statement.read::<String, _>("value").unwrap();
                        get_output(val);
                    }
                },
                "edit" => {
                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let val = statement.read::<String, _>("value").unwrap();
                        get_output(format!("{} {}", env::var("ACT_EDITOR").expect("Missing ACT_EDITOR environment variable."), val));
                    }
                },
                _ => {
                    eprintln!("error: invalid command");
                    print_help();
                },
            }
        },
        _ => print_help()
    }
}

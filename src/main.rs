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
    println!("{}", String::from("Help me.").red())
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
                "ls" => {
                    let query = format!("SELECT * FROM astore;");
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let key = statement.read::<String, _>("key").unwrap().blue();
                        let val = statement.read::<String, _>("value").unwrap().green();
                        println!("{}: {}", key, val)
                    }
                },
                _ => println!("This is not the answer."),
            }
        },
        3.. => {
            let cmd = &args[1];
            let ln = &args[2..].join(" ");

            match &cmd[..] {
                "set" => {
                    let (key, value) = split_first_word(ln);
                    let query = format!("
                        CREATE TABLE IF NOT EXISTS astore (key TEXT, value TEXT);
                        INSERT INTO astore VALUES ('{}', '{}');
                    ", key, value);
                    connection.execute(query).unwrap();
                },
                "get" => {
                    let query = format!("SELECT * FROM astore where key = '{}';", ln);
                    let mut statement = connection.prepare(query).unwrap();
                    while let Ok(State::Row) = statement.next() {
                        let key = statement.read::<String, _>("key").unwrap().blue();
                        let val = statement.read::<String, _>("value").unwrap().green();
                        println!("{}: {}", key, val)
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
                _ => {
                    eprintln!("error: invalid command");
                    print_help();
                },
            }
        },
        _ => print_help()
    }
}

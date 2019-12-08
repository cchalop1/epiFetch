#[macro_use]
extern crate prettytable;
extern crate reqwest;
extern crate dirs;
extern crate structopt;

mod data;
mod header;

use data::{User, Board, Home, ModulesNotes};

use std::fs;
use std::io::{self};
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
use std::path::PathBuf;

const FILE_AUTOLOGIN: &str = "autologin";

#[derive(StructOpt, Debug)]
#[structopt(name = "epiFetch")]
#[allow(non_camel_case_types)]
enum Opt {
    /// Display user information
    user,
    /// Display all current project and for see detail put <id> after project
    project {
        idx: Option<i32>,
    },
    /// Display all your notes
    notes,
    /// Display all your modules
    modules
}


fn main() {
    println!("{}",header::HEADER_EPIFETCH);
    start();
}

fn start() {
    let matches = Opt::from_args();
    let autologin_url: String = load_autologin();

    match matches {
        Opt::user => {
            fetch_user(&autologin_url)
                .print();
        },
        Opt::project {idx} => {
            match idx {
                Some(idx) => {
                    fetch_project(&autologin_url)
                        .print_project_detail(idx, &autologin_url);
                    },
                None => {
                    fetch_project(&autologin_url)
                        .print_projects();
                }
            }
        },
        Opt::notes => {
            fetch_note_modules(&autologin_url)
                .print_notes();
        },
        Opt::modules => {
            fetch_note_modules(&autologin_url)
                .print_modules();
        }
    };
}

fn fetch_project(autologin_url: &String) -> Board {
    let url: String = builder_url(&autologin_url, "");
    let home: Home = reqwest::get(&url[..])
                                .unwrap()
                                .json()
                                .unwrap();
    home.board
}

fn fetch_user(autologin_url: &String) -> User {
    let url: String = builder_url(&autologin_url, "/user");
    reqwest::get(&url[..])
                    .unwrap()
                    .json()
                    .unwrap()
}

fn fetch_note_modules(autologin_url: &String) -> ModulesNotes {
    let user: User = fetch_user(&autologin_url);
    let url: String = builder_url(&autologin_url, &format!("/user/{}/notes/", user.login)[..]);
    reqwest::get(&url[..])
                    .unwrap()
                    .json()
                    .unwrap()
}

fn builder_url(autologin_url: &String, path: &str) -> String {
    format!("{}{}{}", autologin_url, path, "/?format=json")
}

fn get_config_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home config dir /home/username/.config!"),
    }
}

fn load_autologin() -> String {
    let mut input = String::new();
    let text_prompt: String = format!("You can find your autologin at this link: https://intra.epitech.eu/admin/autolog\nWhat is your autologin ?\n");
    let config_file_path: String = format!("{}/{}", get_config_path().to_str().expect("error"), FILE_AUTOLOGIN);

    match fs::read_to_string(&config_file_path) {
        Ok(mut autolog) => {
            autolog.pop();
            autolog
        },
        Err(_) => {
            print!("{}", text_prompt);
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let mut file = File::create(&config_file_path).expect("");
                    file.write_all(input[..].as_bytes()).expect("");
                    input.pop();
                    input
                },
                Err(err) => panic!(format!("Error when readline: {}", err))
            }
        }
    }
}

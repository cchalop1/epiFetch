#[macro_use]
extern crate prettytable;
extern crate crypto;
extern crate dirs;
extern crate reqwest;
extern crate rpassword;
extern crate rustc_serialize;
extern crate structopt;

mod data;
mod header;

use data::{Blih, Board, Home, ModulesNotes, NewRepo, Pass, Repos, User, BlihData};

use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha512;

use rustc_serialize::hex::ToHex;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::path::PathBuf;
use structopt::StructOpt;

const BLIH_URL: &str = "https://blih.epitech.eu";
const FILE_AUTOLOGIN: &str = "autologin";

#[derive(StructOpt, Debug)]
#[structopt(name = "epiFetch")]
#[allow(non_camel_case_types)]
enum Opt {
    /// Display user information
    user,
    /// Display all current project and for see detail put <id> after project
    project { idx: Option<i32> },
    /// Display all your notes
    notes,
    /// Display all your modules
    modules,
    /// Dispaly all repos on blih
    /// Create new Repo on blih and pass option if you want give right to ramasage tek
    repo { repo_name: Option<String> },
}

fn main() {
    println!("{}", header::HEADER_EPIFETCH);
    start();
}

fn start() {
    let matches = Opt::from_args();
    let pass: Pass = load_passwd();

    match matches {
        Opt::user => fetch_user(&pass.autologin).print(),
        Opt::project { idx } => match idx {
            Some(idx) => fetch_project(&pass.autologin).print_project_detail(idx, &pass.autologin),
            None => fetch_project(&pass.autologin).print_projects(),
        },
        Opt::notes => fetch_note_modules(&pass.autologin).print_notes(),
        Opt::modules => fetch_note_modules(&pass.autologin).print_modules(),
        Opt::repo { repo_name } => match repo_name {
            Some(repo_name) => fetch_create_repo(&pass, repo_name),
            None => fetch_repos(&pass).print_repos(),
        },
    };
}

fn fetch_create_repo(pass: &Pass, repo_name: String) {
    let repo = NewRepo {
        name: repo_name,
        repo_type: format!("git"),
    };
     let data: BlihData = BlihData {
        user: format!("{}", &pass.login),
        // update signature with new new data indent
        signature: format!("{}", &pass.passwd),
        data: repo,
    };
    // println!("{:#?}", data);
    let url: String = builder_url_blih(&BLIH_URL, "/repositories");
    let resp = reqwest::Client::new()
        .post(&url[..])
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap())
        .send()
        .unwrap()
        .text()
        .unwrap();
    println!("{}", resp);
}

fn fetch_repos(pass: &Pass) -> Repos {
    let data: Blih = Blih {
        user: format!("{}", &pass.login),
        signature: format!("{}", &pass.passwd),
    };
    let url: String = builder_url_blih(&BLIH_URL, "/repositories");
    reqwest::Client::new()
        .get(&url[..])
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap())
        .send()
        .unwrap()
        .json()
        .unwrap()
}

fn fetch_project(autologin_url: &String) -> Board {
    let url: String = builder_url_autologin(&autologin_url, "");
    let home: Home = reqwest::get(&url[..]).unwrap().json().unwrap();
    home.board
}

fn fetch_user(autologin_url: &String) -> User {
    let url: String = builder_url_autologin(&autologin_url, "/user");
    reqwest::get(&url[..]).unwrap().json().unwrap()
}

fn fetch_note_modules(autologin_url: &String) -> ModulesNotes {
    let user: User = fetch_user(&autologin_url);
    let url: String =
        builder_url_autologin(&autologin_url, &format!("/user/{}/notes/", user.login)[..]);
    reqwest::get(&url[..]).unwrap().json().unwrap()
}

fn builder_url_autologin(autologin_url: &String, path: &str) -> String {
    format!("{}{}{}", autologin_url, path, "/?format=json")
}

fn builder_url_blih(blih_url: &str, path: &str) -> String {
    format!("{}{}", blih_url, path)
}

fn get_config_path() -> PathBuf {
    match dirs::config_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home config dir /home/username/.config!"),
    }
}

fn hash_passwd(raw_passwd: &String) -> String {
    let mut sha = Sha512::new();
    sha.input_str(raw_passwd);
    sha.result_str()
}

fn do_hamxc_login_passwd(passwd: &String, login: &String) -> String {
    let mut hmac = Hmac::new(Sha512::new(), &passwd.as_bytes());
    hmac.input(&login.as_bytes());
    // TODO: test re instet
    hmac.result().code().to_hex()
}

fn create_file_and_prompt_info(config_file_path: String) -> Pass {
    let mut autologin: String = String::new();
    let mut login: String = String::new();

    print!("You can find your autologin at this link: https://intra.epitech.eu/admin/autolog\nWhat is your autologin link ?\n");
    io::stdin().read_line(&mut autologin).unwrap();
    autologin.pop();
    print!("Your epitech login ?\n");
    io::stdin().read_line(&mut login).unwrap();
    login.pop();
    print!("Your epitech password ?\n");
    let mut passwd = rpassword::read_password().unwrap();
    passwd = do_hamxc_login_passwd(&hash_passwd(&passwd), &login);

    let pass: Pass = Pass {
        autologin: autologin,
        login: login,
        passwd: passwd,
    };
    let mut file = File::create(&config_file_path).unwrap();
    file.write_all(serde_json::to_string(&pass).unwrap()[..].as_bytes())
        .unwrap();
    return pass;
}

fn load_passwd() -> Pass {
    let config_file_path: String = format!(
        "{}/{}",
        get_config_path().to_str().expect("Error config folder"),
        FILE_AUTOLOGIN
    );

    match fs::read_to_string(&config_file_path) {
        Ok(autolog) => serde_json::from_str(&autolog).expect("Error file to json"),
        Err(_) => create_file_and_prompt_info(config_file_path),
    }
}

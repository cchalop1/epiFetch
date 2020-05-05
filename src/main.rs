#[macro_use]
extern crate prettytable;
extern crate crypto;
extern crate dirs;
extern crate reqwest;
extern crate rpassword;
extern crate rustc_serialize;
extern crate structopt;

#[macro_use]
extern crate serde_json;

mod data;
mod header;

use data::{Blih, BlihData, BlihResponse, Board, Document, Home, ModulesNotes, Pass, Repos, User};

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
    project {
        idx: Option<i32>,
        dl: Option<String>,
    },
    /// Display all activites
    activity { idx: Option<i32> },
    /// Display all your notes
    notes,
    /// Display all your modules
    modules,
    /// Dispaly all repos on blih
    /// <repo_name> for create
    repo { repo_name: Option<String> },
    /// Set ripo right <repo name> <user> <user_right>
    setacl {
        repo_name: String,
        user: String,
        user_right: String,
    },
    /// enter one token
    token { idx: Option<i32> },
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
        Opt::project { idx, dl } => match (idx, dl) {
            (Some(idx), None) => {
                fetch_home(&pass.autologin).print_project_detail(idx, &pass.autologin)
            }
            (None, Some(_dl)) => println!("pleas give me project id"),
            (Some(idx), Some(dl)) => {
                if dl == "dl" {
                    download_project_file(idx, &pass.autologin)
                }
            }
            (None, None) => fetch_home(&pass.autologin).print_projects(),
        },
        Opt::activity { idx } => match idx {
            Some(idx) => fetch_home(&pass.autologin).print_activity_detail(idx, &pass.autologin),
            None => fetch_home(&pass.autologin).print_activity(),
        },
        Opt::notes => fetch_note_modules(&pass.autologin).print_notes(),
        Opt::modules => fetch_note_modules(&pass.autologin).print_modules(),
        Opt::repo { repo_name } => match repo_name {
            Some(repo_name) => fetch_create_repo(&pass, repo_name),
            None => fetch_repos(&pass).print_repos(),
        },
        Opt::setacl {
            repo_name,
            user,
            user_right,
        } => fetch_right_repo(&pass, repo_name, user, user_right),
        Opt::token { idx } => match idx {
            Some(idx) => println!("{}", idx),
            None => fetch_all_token_open(&pass.autologin),
        },
    };
}

fn json_for_create_repo(name: &String) -> String {
    format!("{{\n    \"name\": \"{}\",\n    \"type\": \"git\"\n}}", name)
}

fn json_for_right_repo(right: &String, user: &String) -> String {
    format!(
        "{{\n    \"acl\": \"{}\",\n    \"user\": \"{}\"\n}}",
        right, user
    )
}

fn fetch_create_repo(pass: &Pass, repo_name: String) {
    let data: BlihData = BlihData {
        user: format!("{}", &pass.login),
        signature: do_hamxc_login_passwd(
            &pass.passwd,
            &pass.login,
            Some(json_for_create_repo(&repo_name)),
        ),
        data: json!({"name": &repo_name, "type": "git"}),
    };
    let url: String = builder_url_blih(&BLIH_URL, "/repositories");
    let resp: BlihResponse = reqwest::Client::new()
        .post(&url[..])
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap())
        .send()
        .unwrap()
        .json()
        .unwrap();
    if let Some(error) = resp.error {
        println!("{}", error);
    }
    if let Some(message) = resp.message {
        println!("{}", message);
        println!("git@git.epitech.eu:/{}/{}", pass.login, repo_name);
        fetch_right_repo(&pass, repo_name, format!("ramassage-tek"), format!("r"));
    }
}

fn fetch_right_repo(pass: &Pass, repo_name: String, user: String, user_right: String) {
    let data: BlihData = BlihData {
        user: format!("{}", &pass.login),
        signature: do_hamxc_login_passwd(
            &pass.passwd,
            &pass.login,
            Some(json_for_right_repo(&user_right, &user)),
        ),
        data: json!({"user": &user, "acl": &user_right}),
    };
    let url_path: &str = &format!("/repository/{}/acls", repo_name)[..];
    let url: String = builder_url_blih(&BLIH_URL, url_path);
    let resp: BlihResponse = reqwest::Client::new()
        .post(&url[..])
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap())
        .send()
        .unwrap()
        .json()
        .unwrap();
    if let Some(error) = resp.error {
        println!("{}", error);
    }
    if let Some(message) = resp.message {
        println!("{}", message);
    }
}

fn fetch_repos(pass: &Pass) -> Repos {
    let data: Blih = Blih {
        user: format!("{}", &pass.login),
        signature: do_hamxc_login_passwd(&pass.passwd, &pass.login, None),
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

fn download_project_file(idx: i32, autologin_url: &String) {
    let url: String = builder_url_autologin(&autologin_url, "");
    let home: Home = reqwest::get(&url[..]).unwrap().json().unwrap();

    match home.board.projets.get(idx as usize) {
        Some(project) => {
            let url_document: String = builder_url_autologin(
                &autologin_url,
                &format!("{}{}", project.title_link, "project/file")[..],
            );
            let documents: Vec<Document> = reqwest::get(&url_document[..]).unwrap().json().unwrap();
            for doc in documents {
                let url_file = format!("https://intra.epitech.eu{}", &doc.fullpath[..]);
                let mut resp = reqwest::get(&url_file[..]).expect("request failed");
                let mut out = File::create(&doc.title[..]).expect("failed to create file");
                io::copy(&mut resp, &mut out).expect("failed to copy content");
            };
        }
        None => return,
    }
}

fn fetch_home(autologin_url: &String) -> Board {
    let url: String = builder_url_autologin(&autologin_url, "");
    let home: Home = reqwest::get(&url[..]).unwrap().json().unwrap();
    home.board
}

fn fetch_all_token_open(autologin_url: &String) {
    let url: String = builder_url_autologin(&autologin_url, "");
    let home: Home = reqwest::get(&url[..]).unwrap().json().unwrap();
    for act in home.board.activites.iter() {
        println!("{:?}", act);
    }
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

fn do_hamxc_login_passwd(passwd: &String, login: &String, data: Option<String>) -> String {
    let mut hmac = Hmac::new(Sha512::new(), &passwd.as_bytes());
    hmac.input(format!("{}", login).as_bytes());
    if let Some(data) = data {
        hmac.input(format!("{}", data).as_bytes());
    }
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
    passwd = hash_passwd(&passwd);

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

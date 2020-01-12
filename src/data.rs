use prettytable::{format, Table};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize,Serialize, Debug)]
pub struct Pass {
    pub autologin: String,
    pub login: String,
    pub passwd: String,
}

#[derive(Deserialize, Debug)]
pub struct Gpa {
    gpa: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub login: String,
    title: String,
    scolaryear: String,
    promo: i32,
    location: String,
    gpa: Vec<Gpa>,
}

#[derive(Deserialize, Debug)]
pub struct Home {
    ip: String,
    pub board: Board,
}

#[derive(Deserialize, Debug)]
pub struct Board {
    projets: Vec<Projet>,
    // activites: Vec<Activites>,
}

#[derive(Deserialize, Debug)]
pub struct Projet {
    title: String,
    title_link: String,
    timeline_start: String,
    timeline_end: String,
    timeline_barre: String,
    date_inscription: Value,
    id_activite: String,
}

#[derive(Deserialize, Debug)]
pub struct Activites {
    title: String,
    module: String,
    module_link: String,
    module_code: String,
    title_link: String,
    timeline_start: String,
    timeline_end: String,
    timeline_barre: String,
    salle: String,
    token: String,
    token_link: String,
}

#[derive(Deserialize, Debug)]
pub struct Module {
    title: String,
    date_ins: String,
    scolaryear: i32,
    grade: String,
    credits: i32,
}

#[derive(Deserialize, Debug)]
pub struct Note {
    title: String,
    titlemodule: String,
    date: String,
    scolaryear: i32,
    final_note: f32,
}

#[derive(Deserialize, Debug)]
pub struct ModulesNotes {
    modules: Vec<Module>,
    notes: Vec<Note>,
}

#[derive(Deserialize, Debug)]
pub struct Repo {
    url: String,
    uuid: String,
}

#[derive(Deserialize, Debug)]
pub struct Repos {
    message: String,
    repositories: HashMap<String, Repo>,
}

impl Repos {
    pub fn print_repos(&self) {
        let mut table = Table::new();
        table.set_format(format_display_table());
        table.add_row(row!["ID", "REPO_NAME"]);
        table.add_row(row!["--", "---------"]);
        for (idx, repo) in self.repositories.keys().enumerate() {
            table.add_row(row![idx, repo]);
        }
        table.printstd();
    }
}

#[derive(Serialize, Debug)]
pub struct Blih {
    pub user: String,
    pub signature: String,
}

#[derive(Serialize, Debug)]
pub struct BlihData {
    pub user: String,
    pub signature: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct BlihResponse {
    pub message: Option<String>,
    pub error: Option<String>,
}

impl User {
    pub fn print(&self) {
        let mut table = Table::new();
        table.set_format(format_display());
        table.add_row(row!["Name: ", self.title]);
        table.add_row(row!["Login: ", self.login]);
        table.add_row(row!["Promo: ", self.promo]);
        table.add_row(row!["Gpa: ", self.gpa[0].gpa]);
        table.add_row(row!["Scolaryear: ", self.scolaryear]);
        table.add_row(row!["Location: ", self.location]);
        table.printstd();
    }
}

fn parce_json_float_to_string(str: &String) -> String {
    let all_elements: Vec<&str> = str.split(".").collect();
    format!("{}", all_elements[0])
}

impl Board {
    pub fn print_projects(&self) {
        let mut table = Table::new();
        table.set_format(format_display_table());
        table.add_row(row!["ID", "PROJECT_NAME", "TIMELINE_BARRE"]);
        table.add_row(row!["--", "------------", "--------------"]);
        for (idx, project) in self.projets.iter().enumerate() {
            let nbr: String = parce_json_float_to_string(&project.timeline_barre);
            table.add_row(row![
                idx,
                project.title,
                format!("|{}|{}%", parce_timeline(&nbr), &nbr)
            ]);
        }
        print!("\n");
        table.printstd();
    }

    pub fn print_project_detail(&self, idx: i32, autologin_url: &String) {
        match self.projets.get(idx as usize) {
            Some(proj) => {
                let mut table = Table::new();
                let nbr: String = parce_json_float_to_string(&proj.timeline_barre);
                table.set_format(format_display());
                table.add_row(row!["Title: ", proj.title]);
                table.add_row(row![
                    "Link: ",
                    format!("{}{}project/", autologin_url, proj.title_link)
                ]);
                table.add_row(row!["Start_Time: ", proj.timeline_start]);
                table.add_row(row!["End_Time: ", proj.timeline_end]);
                table.add_row(row![
                    "Time_Barre: ",
                    format!("|{}|{}%", parce_timeline(&nbr), &nbr)
                ]);
                table.add_row(row!["Date_inscription: ", proj.date_inscription]);
                table.printstd();
            }
            None => panic!("there is no project with this id"),
        }
    }

    // pub fn print_activity(&self) {
    //     let mut table = Table::new();
    //     table.set_format(format_display_table());
    //     table.add_row(row!["ID", "ACTIVITY_NAME", "TIMELINE_BARRE"]);
    //     table.add_row(row!["--", "-------------", "--------------"]);
    //     for (idx, activite) in self.activites.iter().enumerate() {
    //         let nbr: String = parce_json_float_to_string(&activite.timeline_barre);
    //         table.add_row(row![
    //             idx,
    //             activite.title,
    //             format!("|{}|{}%", parce_timeline(&nbr), &nbr)
    //         ]);
    //     }
    //     print!("\n");
    //     table.printstd();
    // }
}

impl ModulesNotes {
    pub fn print_notes(&self) {
        let mut table = Table::new();
        table.set_format(format_display_table());
        table.add_row(row!["TITLE", "MODULES", "DATE", "SCOLARYEAR", "NOTE"]);
        table.add_row(row!["-----", "-------", "----", "----------", "----"]);
        for note in self.notes.iter() {
            table.add_row(row![
                note.title,
                note.titlemodule,
                note.date,
                note.scolaryear,
                note.final_note
            ]);
        }
        print!("\n");
        table.printstd();
    }

    pub fn print_modules(&self) {
        let mut table = Table::new();
        table.set_format(format_display_table());
        table.add_row(row!["TITLE", "DATE", "SCOLARYEAR", "GRADE", "CREDIT"]);
        table.add_row(row!["-----", "----", "----------", "-----", "------"]);
        for mode in self.modules.iter() {
            table.add_row(row![
                mode.title,
                mode.date_ins,
                mode.scolaryear,
                mode.grade,
                mode.credits
            ]);
        }
        print!("\n");
        table.printstd();
    }
}

fn format_display() -> format::TableFormat {
    format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .separators(
            &[format::LinePosition::Top, format::LinePosition::Bottom],
            format::LineSeparator::new(' ', ' ', ' ', ' '),
        )
        .padding(0, 0)
        .build()
}

fn format_display_table() -> format::TableFormat {
    format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[format::LinePosition::Top, format::LinePosition::Bottom],
            format::LineSeparator::new('-', '+', '+', '+'),
        )
        .padding(2, 2)
        .build()
}

fn parce_timeline(nbr: &String) -> String {
    let max: i32 = nbr.parse().unwrap();

    (0..10)
        .map(|i| match i * 10 < max {
            true => 35 as char,
            false => 32 as char,
        })
        .collect()
}

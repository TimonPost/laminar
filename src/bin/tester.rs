#[macro_use]
extern crate clap;
extern crate laminar;

use std::process::exit;

use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(_) = matches.subcommand_matches("server") {
        run_server();
        exit(0);
    }

    if let Some(_) = matches.subcommand_matches("client") {
        run_client();
        exit(0);
    }
}

fn run_server() {
    println!("Ran server!");
}

fn run_client() {
    println!("Ran client!");
}

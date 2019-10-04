extern crate pythia;
extern crate clap;
use clap::{Arg, App, SubCommand};

use pythia::{
    get_manifest,
    get_trace,
    make_decision,
};

fn main() {
    let matches = App::new("Pythia")
        .version("1.0")
        .author("Emre Ates <ates@bu.edu>")
        .subcommand(SubCommand::with_name("manifest")
            .arg(Arg::with_name("manifest_file")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("get_trace")
            .arg(Arg::with_name("trace_id")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("diagnose"))
        .get_matches();
    match matches.subcommand() {
        ("manifest", Some(matches)) => {
            get_manifest(matches.value_of("manifest_file").unwrap());
        },
        ("get_trace", Some(matches)) => {
            get_trace(matches.value_of("trace_id").unwrap());
        },
        ("diagnose", Some(_)) => {
            make_decision();
        },
        _ => panic!("Must provide a subcommand, see --help for commands")
    };
}

#[macro_use]
extern crate clap;
use clap::App;

mod jenkins_ctl;

fn main() {
    let yml = load_yaml!("cmd.yml");
    let matches = App::from_yaml(yml).get_matches();

    let ret = match matches.subcommand() {
        ("jenkins", Some(args)) => jenkins_ctl::process(args),
        _ => {
            eprintln!("Unknwown subcommand");
            Ok(())
        }
    };

    if let Err(err) = ret {
        eprintln!("{}", err)
    }
}

use std::fs;
use std::thread::sleep;
use std::time::Duration;

use clap::ArgMatches;
use serde::Deserialize;

use jenkins_ci::{Job, ParameterList};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct Config {
    url: String,
    job: String,
    user: String,
    token: String,
    with_crumb: bool,
}

fn download(job: &Job, matches: &ArgMatches) -> Result<()> {
    let dest = matches.value_of("destination").unwrap_or(".");
    let alt_name = matches.value_of("alt-name");

    if matches.is_present("from-build") {
        job.get_build(matches.value_of("from-build").unwrap())?
            .download_artifact(matches.value_of("filter").unwrap(), dest, alt_name)?
    } else if matches.is_present("from-ws") {
        let as_file = if let Some(val) = matches.value_of("as-file") {
            val.parse::<bool>().unwrap_or(false)
        } else {
            false
        };

        job.download_from_workspace(
            matches.value_of("from-ws").unwrap(),
            dest,
            alt_name,
            as_file,
        )?
    }

    Ok(())
}

fn trigger(job: &Job, matches: &ArgMatches) -> Result<()> {
    if matches.is_present("params") {
        let vals: Vec<&str> = matches.values_of("params").unwrap().collect();
        let mut params = ParameterList::new();
        let kv: Vec<(&str, &str)> = vals
            .iter()
            .map(|val| val.split_once('='))
            .filter(|opt| opt.is_some())
            .map(|opt| opt.unwrap())
            .collect();

        params.add_list(&kv);
        job.trigger_build_with_params(&params)?;
    } else if matches.is_present("params-from-json") {
        let content = fs::read_to_string(matches.value_of("params-from-json").unwrap())?;
        let params = ParameterList::from_string(&content);

        job.trigger_build_with_params(&params)?
    } else {
        job.trigger_build()?
    }

    if matches.is_present("wait-for-completion") {
        let delay = matches
            .value_of("delay")
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);
        sleep(Duration::from_secs(delay));
    }
    Ok(())
}

fn info(job: &Job, matches: &ArgMatches) -> Result<()> {
    let id: Vec<&str> = matches.values_of("build-id").unwrap().collect();

    let build = job.get_build(id[0])?;
    println!("{}", build);
    Ok(())
}

pub fn process(matches: &ArgMatches) -> Result<()> {
    let job = if let Some(file) = matches.value_of("config") {
        let conf: Config = serde_json::from_str(&fs::read_to_string(file)?)?;
        Job::new(
            &conf.url,
            &conf.job,
            &conf.user,
            &conf.token,
            conf.with_crumb,
        )?
    } else {
        todo!("implémenter les valeurs via des options de ligne de commande")
    };

    match matches.subcommand() {
        ("download", Some(args)) => download(&job, args)?,
        ("trigger", Some(args)) => trigger(&job, args)?,
        ("info", Some(args)) => info(&job, args)?,
        _ => {
            eprintln!("Unknwown Jenkins API subcommand");
            return Ok(());
        }
    }

    Ok(())
}

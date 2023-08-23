// Copyright 2017 Jeremy Wall <jeremy@marzhillstudios.com>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// runwhen - A utility that runs commands on user defined triggers.
#[macro_use]
extern crate clap;
extern crate glob;
extern crate humantime;
extern crate notify;

use std::{process, str::FromStr};

mod error;
mod events;
mod exec;
mod file;
mod timer;
mod traits;

use events::WatchEventType;
use exec::ExecProcess;
use file::FileProcess;
use timer::TimerProcess;
use traits::Process;

#[rustfmt::skip]
fn do_flags() -> clap::ArgMatches {
    clap::command!()
        .version(crate_version!())
        .author(crate_authors!())
        .about("Runs a command on user defined triggers.")
        .arg(arg!(-c --cmd).takes_value(true).help("The command to run on the trigger"))
        .arg(arg!(-e --env ...).takes_value(true).help("Set of environment variables to set for the command"))
        .subcommand(
            clap::Command::new("watch")
                .about("Trigger that fires when a file or directory changes.")
                .arg(
                    arg!(-f --file).name("file")
                        .takes_value(true).help("File or directory to watch for changes"),
                )
                .arg(
                    arg!(-e --exclude).name("exclude")
                        .takes_value(true).help("path names to skip when watching. Specified in unix glob format."),
                )
                .arg(arg!(--touch).name("filetouch").help("Use file or directory timestamps to monitor for changes."))
            .arg(arg!(--poll).name("poll").takes_value(true).value_parser(value_parser!(humantime::Duration)).help("Duration of time between polls")))
        .subcommand(
            clap::Command::new("timer")
                .about("Run command on a timer")
                .arg(arg!(-t --duration).takes_value(true).value_parser(value_parser!(humantime::Duration)).help("Duration between runs"))
                .arg(arg!(-n --repeat).value_parser(value_parser!(u32)).help("Number of times to run before finishing")))
        .subcommand(
            clap::Command::new("success")
            .about("Run a command when a test command succeeds")
            .arg(arg!(--if).value_parser(value_parser!(String)).help("The command to run and check for success on"))
            .arg(arg!(--not).help("Negate the success of the command"))
            .arg(arg!(--poll).value_parser(value_parser!(humantime::Duration)).help("Duration of time between poll")))
        .get_matches()
}

fn main() {
    let app = do_flags();
    // Unwrap because this flag is required.
    let cmd = app.value_of("cmd").expect("cmd flag is required");
    let mut maybe_env = None;
    if let Some(env_values) = app.values_of("env") {
        let mut env_vec = Vec::new();
        for v in env_values {
            env_vec.push(v.to_string());
        }
        maybe_env = Some(env_vec);
    }

    let mut proc: Box<dyn Process> = if let Some(matches) = app.subcommand_matches("watch") {
        let file = match matches.values_of("file") {
            Some(v) => v.collect(),
            // The default is our current directory
            None => vec!["."],
        };
        let mut method = WatchEventType::Changed;
        if matches.is_present("filetouch") {
            method = WatchEventType::Touched;
        }
        let duration = match matches.get_one::<humantime::Duration>("poll") {
            Some(d) => Some((*d).into()),
            None => None,
        };
        let exclude = match matches.values_of("exclude") {
            Some(vr) => Some(vr.collect()),
            None => None,
        };
        println!("Enforcing a poll time of {:?}", duration);
        Box::new(FileProcess::new(
            cmd, maybe_env, file, exclude, method, duration,
        ))
    } else if let Some(matches) = app.subcommand_matches("timer") {
        // TODO(jwall): This should use cancelable commands.
        // Unwrap because this flag is required.
        let duration = matches
            .get_one::<humantime::Duration>("duration")
            .expect("duration flag is required")
            .clone();
        let max_repeat = matches.get_one::<u32>("repeat").cloned();
        Box::new(TimerProcess::new(cmd, maybe_env, *duration, max_repeat))
    } else if let Some(matches) = app.subcommand_matches("success") {
        // unwrap because this is required.
        let ifcmd = matches.value_of("ifcmd").expect("ifcmd flag is required");
        let negate = matches.is_present("not");
        let duration = *matches
            .get_one::<humantime::Duration>("poll")
            .cloned()
            .unwrap_or(humantime::Duration::from_str("5s").unwrap());
        Box::new(ExecProcess::new(ifcmd, cmd, negate, maybe_env, duration))
    } else {
        println!("You must specify a subcommand.");
        process::exit(1)
    };
    match proc.run() {
        Ok(_) => return,
        Err(err) => {
            println!("{0}", err);
            process::exit(1)
        }
    }
}

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
extern crate humantime;
extern crate notify;
extern crate subprocess;

use std::process;
use std::str::FromStr;

mod traits;
mod file;
mod timer;
mod error;
mod events;
mod exec;

use traits::Process;
use file::FileProcess;
use timer::TimerProcess;
use exec::ExecProcess;
use events::WatchEventType;

fn do_flags<'a>() -> clap::ArgMatches<'a> {
    clap_app!(
        runwhen =>
            (version: crate_version!())
            (author: crate_authors!())
            (about: "Runs a command on user defined triggers.")
            (@arg cmd: -c --cmd +required +takes_value "Command to run on supplied triggers")
            (@subcommand watch =>
             (about: "Trigger that fires when a file or directory changes.")
             // TODO(jeremy): We need to support filters
             (@arg file: -f --file +takes_value
              "File/Directory to watch. (default current working directory)")
             (@arg filetouch: --touch
              "Watches for attribute modifications as well as content changes.")
             (@arg wait: --poll +takes_value
              "How frequently to poll for events (default 5s)")
            )
            (@subcommand timer =>
             (about: "Trigger that fires on a timer.")
             (@arg duration: -t --duration +required +takes_value
              "Defines timer frequency.")
             (@arg repeat: -n --repeat +takes_value
              "Defines an optional max number times to run on repeat.")
            )
            (@subcommand success =>
             (about: "Trigger that fires if a command runs successful.")
             (@arg ifcmd: --if +required +takes_value
              "The command to test for successful exit from")
             (@arg wait: --poll +takes_value
              "How frequently to test command (default 5s)")
            )
    )
        .get_matches()
}

fn main() {
    let app = do_flags();
    // Unwrap because this flag is required.
    let cmd = app.value_of("cmd").expect("cmd flag is required");
    let mut process: Option<Box<Process>> = None;
    if let Some(matches) = app.subcommand_matches("watch") {
        // Unwrap because this flag is required.
        let file = matches.value_of("file").unwrap_or(".");
        let mut method = WatchEventType::Changed;
        if matches.is_present("filetouch") {
            method = WatchEventType::Touched;
        }
        let poll = matches.value_of("poll").unwrap_or("5s");
        let dur = humantime::parse_duration(poll).expect("Invalid poll value.");
        process = Some(Box::new(FileProcess::new(cmd, file, method, dur)));
    } else if let Some(matches) = app.subcommand_matches("timer") {
        // Unwrap because this flag is required.
        let dur = humantime::parse_duration(matches.value_of("duration")
            .expect("duration flag is required"));
        match dur {
            Ok(duration) => {
                let max_repeat = if let Some(val) = matches.value_of("repeat") {
                    match u32::from_str(val) {
                        Ok(n) => Some(n),
                        Err(e) => {
                            println!("Invalid --repeat value {}", e);
                            println!("{}", matches.usage());
                            process::exit(1)
                        }
                    }
                } else {
                    None
                };
                process = Some(Box::new(TimerProcess::new(cmd, duration, max_repeat)));
            }
            Err(msg) => {
                println!("Malformed duration {:?}", msg);
                process::exit(1);
            }
        }
    } else if let Some(matches) = app.subcommand_matches("success") {
        // unwrap because this is required.
        let ifcmd = matches.value_of("ifcmd").expect("ifcmd flag is required");
        let dur = humantime::parse_duration(matches.value_of("poll").unwrap_or("5s"));
        process = match dur {
            Ok(duration) => Some(Box::new(ExecProcess::new(ifcmd, cmd, duration))),
            Err(msg) => {
                println!("Malformed poll {:?}", msg);
                process::exit(1)
            }
        }
    }
    match process {
        Some(process) => {
            match process.run() {
                Ok(_) => return,
                Err(err) => {
                    println!("{0}", err);
                    process::exit(1)
                }
            }
        }
        None => {
            println!("You must specify a subcommand.");
            process::exit(1)
        }
    }
}

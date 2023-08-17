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
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use glob;
use notify::{watcher, RecursiveMode, Watcher};

use error::CommandError;
use events::WatchEventType;
use exec::CancelableProcess;
use traits::Process;

pub struct FileProcess<'a> {
    cmd: &'a str,
    env: Option<Vec<String>>,
    files: Vec<&'a str>,
    exclude: Option<Vec<&'a str>>,
    method: WatchEventType,
    poll: Option<Duration>,
}

impl<'a> FileProcess<'a> {
    pub fn new(
        cmd: &'a str,
        env: Option<Vec<String>>,
        file: Vec<&'a str>,
        exclude: Option<Vec<&'a str>>,
        method: WatchEventType,
        poll: Option<Duration>,
    ) -> FileProcess<'a> {
        FileProcess {
            cmd,
            env,
            method,
            poll,
            exclude,
            files: file,
        }
    }
}

fn watch_for_change_events(
    ch: Receiver<()>,
    cmd: String,
    env: Option<Vec<String>>,
    poll: Option<Duration>,
) {
    let copied_env = env.and_then(|v| {
        Some(
            v.iter()
                .cloned()
                .map(|s| String::from(s))
                .collect::<Vec<String>>(),
        )
    });
    let mut exec = CancelableProcess::new(&cmd, copied_env);
    println!("Spawning command");
    exec.spawn().expect("Failed to start command");
    println!("Starting watch loop");
    let mut poll_time = Instant::now();
    let mut changed = false;
    println!("Waiting for first change event");
    let _ = ch.recv().expect("Channel was closed!!!");
    loop {
        // Wait our requisit number of seconds
        if let Some(poll) = poll {
            println!("We have a poll time {:?}", poll);
            // If our time has passed and the value has changed then run our step immediately
            if changed && Instant::now().duration_since(poll_time) >= poll {
                println!(
                    "Recieved changed event and waited: {:?}",
                    Instant::now().duration_since(poll_time)
                );
                if !run_loop_step(&mut exec) {
                    println!("Failed to start command");
                }
                changed = false;
                continue;
            }
            println!("Waiting for next change event");
            let _ = ch.recv().expect("Channel was closed!!!");
            changed = true;
            poll_time = Instant::now();
            continue;
        } else {
            println!("We do not have a poll time");
            println!("Waiting for next change event");
            let _ = ch.recv().expect("Channel was closed!!!");
            if !run_loop_step(&mut exec) {
                println!("Failed to start command");
            }
        }
        poll_time = Instant::now();
        changed = false;
    }
}

fn run_loop_step(exec: &mut CancelableProcess) -> bool {
    // We always want to check on our process each iteration of the loop.
    // set signal to false so we won't trigger on the
    // next loop iteration unless we recieved more events.
    // On a true signal we want to start or restart our process.
    println!("Restarting process");
    if let Err(err) = exec.reset() {
        println!("{:?}", err);
        return false;
    }
    return true;
}

fn wait_for_fs_events(
    ch: Sender<()>,
    method: WatchEventType,
    files: &Vec<&str>,
    excluded: &Option<Vec<&str>>,
) -> Result<(), CommandError> {
    // Notify requires a channel for communication.
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    for file in files {
        // NOTE(jwall): this is necessary because notify::fsEventWatcher panics
        // if the path doesn't exist. :-(
        if !Path::new(*file).exists() {
            return Err(CommandError::new(
                format!("No such path! {0}", *file).to_string(),
            ));
        }
        watcher.watch(*file, RecursiveMode::Recursive)?;
        println!("Watching {:?}", *file);
    }
    let mut patterns = Vec::new();
    if let Some(exclude) = excluded {
        for ef in exclude.iter() {
            patterns.push(glob::Pattern::new(*ef).expect("Invalid path pattern"));
            //patterns.push(ef.clone());
            println!("Added pattern {:?}", patterns.iter().last());
        }
    }
    loop {
        let evt: WatchEventType = match rx.recv() {
            Ok(event) => {
                // TODO(jwall): Filter this based on the exclude pattern
                if let Some(f) = crate::events::get_file(&event) {
                    for pat in patterns.iter() {
                        println!("Testing pattern {:?} against {:?}", pat, f);
                        if pat.matches_path(&f) {
                            println!("Excluding: {:?}", f);
                            continue;
                        }
                    }
                }
                WatchEventType::from(event)
            }
            Err(e) => {
                println!("Watch Error: {}", e);
                WatchEventType::Error
            }
        };
        match evt {
            WatchEventType::Ignore | WatchEventType::Error => {
                // We ignore these.
                //println!("Event: Ignore");
            }
            WatchEventType::Touched => {
                if method == WatchEventType::Touched {
                    ch.send(()).unwrap();
                }
            }
            WatchEventType::Changed => {
                ch.send(()).unwrap();
            }
        }
    }
}

impl<'a> Process for FileProcess<'a> {
    fn run(&mut self) -> Result<(), CommandError> {
        // TODO(jeremy): Is this sufficent or do we want to ignore
        // any events that come in while the command is running?
        let (tx, rx) = channel();
        thread::spawn({
            let cmd = self.cmd.to_string();
            let env = self.env.clone();
            let poll = self.poll.clone();
            move || {
                watch_for_change_events(rx, cmd, env, poll);
            }
        });
        wait_for_fs_events(tx, self.method.clone(), &self.files, &self.exclude)?;
        Ok(())
    }
}

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
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{watcher, RecursiveMode, Watcher};

use error::CommandError;
use events::WatchEventType;
use exec::CancelableProcess;
use traits::Process;

pub struct FileProcess<'a> {
    cmd: &'a str,
    env: Option<Vec<String>>,
    files: Vec<&'a str>,
    method: WatchEventType,
    poll: Duration,
}

impl<'a> FileProcess<'a> {
    pub fn new(
        cmd: &'a str,
        env: Option<Vec<String>>,
        file: Vec<&'a str>,
        method: WatchEventType,
        poll: Duration,
    ) -> FileProcess<'a> {
        FileProcess {
            cmd,
            env,
            method,
            poll,
            files: file,
        }
    }
}

fn spawn_runner_thread(
    lock: Arc<Mutex<bool>>,
    cmd: String,
    env: Option<Vec<String>>,
    poll: Duration,
) {
    let copied_env = env.and_then(|v| {
        Some(
            v.iter()
                .cloned()
                .map(|s| String::from(s))
                .collect::<Vec<String>>(),
        )
    });
    thread::spawn(move || {
        let mut exec = CancelableProcess::new(&cmd, copied_env);
        exec.spawn().expect("Failed to start command");
        loop {
            // Wait our requisit number of seconds
            thread::sleep(poll);
            // Default to not running the command.
            if !run_loop_step(lock.clone(), &mut exec) {
                exec.reset().expect("Failed to start command");
            }
        }
    });
}

fn run_loop_step(lock: Arc<Mutex<bool>>, exec: &mut CancelableProcess) -> bool {
    match lock.lock() {
        Ok(mut signal) => {
            // We always want to check on our process each iteration of the loop.
            if let Err(err) = exec.check() {
                println!("{:?}", err);
                return false;
            }
            if *signal {
                // set signal to false so we won't trigger on the
                // next loop iteration unless we recieved more events.
                *signal = false;
                // On a true signal we want to start or restart our process.
                if let Err(err) = exec.reset() {
                    println!("{:?}", err);
                    return false;
                }
            }
            return true;
        }
        Err(err) => {
            println!("Unexpected error; {}", err);
            return false;
        }
    }
}

fn wait_for_fs_events(
    lock: Arc<Mutex<bool>>,
    method: WatchEventType,
    files: &Vec<&str>,
) -> Result<(), CommandError> {
    // Notify requires a channel for communication.
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    // TODO(jwall): Better error handling.
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
    loop {
        let evt: WatchEventType = match rx.recv() {
            Ok(event) => WatchEventType::from(event),
            Err(_) => WatchEventType::Error,
        };
        match evt {
            WatchEventType::Ignore => {
                // We ignore this one.
            }
            WatchEventType::Error => {
                // We log this one.
            }
            WatchEventType::Touched => {
                if method == WatchEventType::Touched {
                    let mut signal = lock.lock().unwrap();
                    *signal = true;
                } else {
                    println!("Ignoring touched event");
                }
            }
            WatchEventType::Changed => match lock.lock() {
                Ok(mut signal) => *signal = true,
                Err(err) => {
                    println!("Unexpected error; {}", err);
                    return Ok(());
                }
            },
        }
    }
}

impl<'a> Process for FileProcess<'a> {
    fn run(&mut self) -> Result<(), CommandError> {
        // TODO(jeremy): Is this sufficent or do we want to ignore
        // any events that come in while the command is running?
        let lock = Arc::new(Mutex::new(false));
        spawn_runner_thread(
            lock.clone(),
            self.cmd.to_string(),
            self.env.clone(),
            self.poll,
        );
        wait_for_fs_events(lock, self.method.clone(), &self.files)
    }
}

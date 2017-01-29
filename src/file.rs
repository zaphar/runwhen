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
use std::thread;
use std::sync::{Arc,Mutex};
use std::time::Duration;
use std::path::Path;
use std::sync::mpsc::channel;

use notify::{Watcher,RecursiveMode,watcher};

use traits::Process;
use error::CommandError;
use events::WatchEventType;
use exec::run_cmd;

pub struct FileProcess<'a> {
    cmd: &'a str,
    file: &'a str,
    method: WatchEventType,
    poll: Duration,
}

impl<'a> FileProcess<'a> {
    pub fn new(cmd: &'a str, file: &'a str, method: WatchEventType, poll: Duration) -> FileProcess<'a> {
        FileProcess{ cmd: cmd, file: file, method: method, poll: poll}
    }
}

fn spawn_runner_thread(lock: Arc<Mutex<bool>>, cmd: String, poll: Duration) {
    thread::spawn(move || {
        loop {
            // Wait our requisit number of seconds
            thread::sleep(poll);
            // Default to not running the command.
            let mut signal = lock.lock().unwrap();
            if *signal {
                // set signal to false so we won't trigger on the
                // next loop iteration unless we recieved more events.
                *signal = false;
                // Run our command!
                if let Err(err) = run_cmd(&cmd) {
                    println!("{:?}", err)
                }
            }
        }
    });
}

fn wait_for_fs_events(lock: Arc<Mutex<bool>>, method: WatchEventType, file: &str) -> Result<(), CommandError> {
    // Notify requires a channel for communication.
    let (tx, rx) = channel();
    let mut watcher = try!(watcher(tx, Duration::from_secs(1)));
    // TODO(jwall): Better error handling.
    try!(watcher.watch(file, RecursiveMode::Recursive));
    println!("Watching {:?}", file);
    loop {
        let evt: WatchEventType = match rx.recv() {
            Ok(event) => {
                WatchEventType::from(event)
            },
            Err(_) => {
                WatchEventType::Error
            }
        };
        match evt {
            WatchEventType::Ignore => {
                // We ignore this one.
            },
            WatchEventType::Error => {
                // We log this one.
            },
            WatchEventType::Touched => {
                if method == WatchEventType::Touched {
                    let mut signal = lock.lock().unwrap();
                    *signal = true;
                }
            },
            WatchEventType::Changed => {
                let mut signal = lock.lock().unwrap();
                *signal = true;
            }
        }
    }
}

impl<'a> Process for FileProcess<'a> {
    fn run(&self) -> Result<(), CommandError> {
    // NOTE(jwall): this is necessary because notify::fsEventWatcher panics
    // if the path doesn't exist. :-(
    if !Path::new(self.file).exists() {
        return Err(CommandError::new(format!("No such path! {0}", self.file).to_string()))
    }
    // TODO(jeremy): Is this sufficent or do we want to ignore
    // any events that come in while the command is running?
    let lock = Arc::new(Mutex::new(false));
    spawn_runner_thread(lock.clone(), self.cmd.to_string(), self.poll);
    wait_for_fs_events(lock, self.method.clone(), self.file)
    }
}

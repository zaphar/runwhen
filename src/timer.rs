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
use std::time::Duration;

use error::CommandError;
use exec::run_cmd;
use traits::Process;

pub struct TimerProcess<'a> {
    cmd: &'a str,
    env: Option<Vec<&'a str>>,
    poll_duration: Duration,
    max_repeat: Option<u32>,
}

impl<'a> TimerProcess<'a> {
    pub fn new(
        cmd: &'a str,
        env: Option<Vec<&'a str>>,
        poll_duration: Duration,
        max_repeat: Option<u32>,
    ) -> TimerProcess<'a> {
        TimerProcess {
            cmd: cmd,
            env: env,
            poll_duration: poll_duration,
            max_repeat: max_repeat,
        }
    }
}

impl<'a> Process for TimerProcess<'a> {
    fn run(&self) -> Result<(), CommandError> {
        let mut counter = 0;
        loop {
            if self.max_repeat.is_some() && counter >= self.max_repeat.unwrap() {
                return Ok(());
            }
            if let Err(err) = run_cmd(self.cmd, &self.env) {
                println!("{:?}", err)
            }
            thread::sleep(self.poll_duration);
            if self.max_repeat.is_some() {
                counter += 1
            }
        }
    }
}

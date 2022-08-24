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

use exec::CancelableProcess;
use error::CommandError;
use traits::Process;

pub struct TimerProcess {
    cmd: CancelableProcess,
    poll_duration: Duration,
    max_repeat: Option<u32>,
}

impl TimerProcess {
    pub fn new(
        cmd: &str,
        env: Option<Vec<String>>,
        poll_duration: Duration,
        max_repeat: Option<u32>,
    ) -> TimerProcess {
    let cmd = CancelableProcess::new(cmd, env);
        TimerProcess {
            cmd,
            poll_duration,
            max_repeat,
        }
    }
}

impl Process for TimerProcess {
    fn run(&mut self) -> Result<(), CommandError> {
        let mut counter = 0;
        loop {
            if self.max_repeat.is_some() && counter >= self.max_repeat.unwrap() {
                return Ok(());
            }
            if let Err(err) = self.cmd.block() {
                println!("{:?}", err)
            }
            thread::sleep(self.poll_duration);
            if self.max_repeat.is_some() {
                counter += 1
            }
        }
    }
}

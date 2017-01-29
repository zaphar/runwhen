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

use subprocess::{Exec,PopenError,ExitStatus};

use traits::Process;
use error::CommandError;

pub fn run_cmd(cmd: &str) -> Result<(), PopenError> {
    Exec::shell(cmd).join()?;
    Ok(())
}

fn is_cmd_success(cmd: &str) -> bool {
    match Exec::shell(cmd).join() {
        Ok(ExitStatus::Exited(code)) => code == 0,
        _ => false,
    }
}

pub struct ExecProcess<'a> {
    test_cmd: &'a str,
    cmd: &'a str,
    poll: Duration,
}

impl<'a> ExecProcess<'a> {
    pub fn new(test_cmd: &'a str, cmd: &'a str, poll: Duration) -> ExecProcess<'a> {
        ExecProcess{test_cmd: test_cmd, cmd: cmd, poll: poll}
    }
}

impl<'a> Process for ExecProcess<'a> {
    fn run(&self) -> Result<(), CommandError> {
        loop {
            if is_cmd_success(self.test_cmd) {
                if let Err(err) = run_cmd(self.cmd) {
                    println!("{:?}", err)
                }
            }
            thread::sleep(self.poll);
        }
    }
}

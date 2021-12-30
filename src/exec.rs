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

use std::process::{Command, Stdio};

use error::CommandError;
use traits::Process;

fn env_var_to_tuple(var: &str) -> (String, String) {
    let mut vs = var.split('=');
    if let Some(name) = vs.next() {
        return match vs.next() {
            Some(val) => (String::from(name), String::from(val)),
            None => (String::from(name), "".to_string()),
        };
    }
    ("".to_string(), "".to_string())
}

pub fn run_cmd(cmd: &str, env: &Option<Vec<&str>>) -> Result<i32, CommandError> {
    let args = cmd
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();
    if args.len() < 1 {
        return Err(CommandError::new("Empty command string passed in"));
    }
    let mut exec = Command::new(args[0]);
    if args.len() > 1 {
        exec.args(&args[1..]);
    }
    exec.stdout(Stdio::inherit());
    exec.stderr(Stdio::inherit());
    if let &Some(ref env_vars) = env {
        for var in env_vars {
            let tpl = env_var_to_tuple(var);
            exec.env(tpl.0, tpl.1);
        }
    }
    return match exec.output() {
        Ok(out) => match out.status.code() {
            Some(val) => Ok(val),
            None => Ok(0),
        },
        // TODO(jeremy): We should not swallow this error.
        Err(_) => Err(CommandError::new("Error running command")),
    };
}

fn is_cmd_success(cmd: &str, env: Option<Vec<&str>>) -> bool {
    match run_cmd(cmd, &env) {
        Ok(code) => code == 0,
        _ => false,
    }
}

pub struct ExecProcess<'a> {
    test_cmd: &'a str,
    negate: bool,
    cmd: &'a str,
    env: Option<Vec<&'a str>>,
    poll: Duration,
}

impl<'a> ExecProcess<'a> {
    pub fn new(
        test_cmd: &'a str,
        cmd: &'a str,
        negate: bool,
        env: Option<Vec<&'a str>>,
        poll: Duration,
    ) -> ExecProcess<'a> {
        ExecProcess {
            test_cmd: test_cmd,
            negate: negate,
            cmd: cmd,
            env: env,
            poll: poll,
        }
    }
}

impl<'a> Process for ExecProcess<'a> {
    fn run(&self) -> Result<(), CommandError> {
        loop {
            // TODO(jwall): Should we set the environment the same as the other command?
            let test_result = is_cmd_success(self.test_cmd, None);
            if (test_result && !self.negate) || (!test_result && self.negate) {
                if let Err(err) = run_cmd(self.cmd, &self.env) {
                    println!("{:?}", err)
                }
            }
            thread::sleep(self.poll);
        }
    }
}

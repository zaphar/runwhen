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
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

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

pub struct CancelableProcess {
    cmd: String,
    env: Option<Vec<String>>,
    exec: Option<Command>,
    handle: Option<Child>,
}

impl CancelableProcess {
    pub fn new(cmd: &str, env: Option<Vec<String>>) -> Self {
        Self {
            cmd: cmd.to_string(),
            env,
            exec: None,
            handle: None,
        }
    }

    fn create_command(cmd: &str, env: &Option<Vec<String>>) -> Result<Command, CommandError> {
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
        return Ok(exec);
    }

    pub fn block(&mut self) -> Result<i32, CommandError> {
        if let Some(ref mut handle) = self.handle {
            let code = handle.wait()?.code().unwrap_or(0);
            self.exec = None;
            self.handle = None;
            Ok(code)
        } else {
            let mut exec = Self::create_command(&self.cmd, &self.env)?;
            return match exec.output() {
                Ok(out) => match out.status.code() {
                    Some(val) => Ok(val),
                    None => Ok(0),
                },
                // TODO(jeremy): We should not swallow this error.
                Err(_) => Err(CommandError::new("Error running command")),
            };
        }
    }

    pub fn is_success(&mut self) -> bool {
        match self.block() {
            Ok(code) => code == 0,
            _ => false,
        }
    }

    // NOTE(jwall): We want to actually use this some time when we figure out if it can be made to not block or not.
    #[allow(dead_code)]
    pub fn check(&mut self) -> Result<Option<i32>, CommandError> {
        Ok(match self.handle {
            // TODO(jwall): This appears to block the thread despite the documenation. Figure out if this is fixable or not.
            Some(ref mut h) => match h.try_wait()? {
                Some(status) => Some(status.code().unwrap_or(0)),
                None => Some(h.wait()?.code().unwrap_or(0)),
            },
            None => None,
        })
    }

    pub fn spawn(&mut self) -> Result<(), CommandError> {
        let mut exec = Self::create_command(&self.cmd, &self.env)?;
        let handle = exec.spawn()?;
        self.exec = Some(exec);
        self.handle = Some(handle);
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), CommandError> {
        if let Some(ref mut h) = self.handle {
            let _ = h.kill();
        }

        self.exec = None;
        self.handle = None;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), CommandError> {
        self.cancel()?;
        self.spawn()?;
        Ok(())
    }
}

// TODO(jwall): Make these CancelableProcess instead.
pub struct ExecProcess {
    test_cmd: CancelableProcess,
    negate: bool,
    cmd: CancelableProcess,
    poll: Duration,
}

impl ExecProcess {
    pub fn new(
        test_cmd: &str,
        cmd: &str,
        negate: bool,
        env: Option<Vec<String>>,
        poll: Duration,
    ) -> ExecProcess {
        let test_cmd = CancelableProcess::new(test_cmd, None);
        let cmd = CancelableProcess::new(cmd, env);
        ExecProcess {
            test_cmd,
            negate,
            cmd,
            poll,
        }
    }

    fn run_loop_step(&mut self) {
        let test_result = self.test_cmd.is_success();
        if (test_result && !self.negate) || (!test_result && self.negate) {
            if let Err(err) = self.cmd.block() {
                println!("{:?}", err)
            }
        }
    }
}

impl Process for ExecProcess {
    fn run(&mut self) -> Result<(), CommandError> {
        loop {
            // TODO(jwall): Should we set the environment the same as the other command?
            self.run_loop_step();
            thread::sleep(self.poll);
        }
    }
}

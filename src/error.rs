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
use std::fmt;
use std::io;

use notify;

#[derive(Debug)]
pub struct CommandError {
    msg: String,
}

impl CommandError {
    pub fn new<S: Into<String>>(msg: S) -> CommandError {
        CommandError { msg: msg.into() }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}

impl From<notify::Error> for CommandError {
    fn from(e: notify::Error) -> CommandError {
        CommandError::new(format!("{}", e))
    }
}

impl From<io::Error> for CommandError {
    fn from(e: io::Error) -> CommandError {
        CommandError::new(format!("IO: {}", e))
    }
}

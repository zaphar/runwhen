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
use notify::DebouncedEvent;

#[derive(PartialEq, Clone)]
pub enum WatchEventType {
    Touched,
    Changed,
    Error,
    Ignore,
}

pub fn get_file(evt: &DebouncedEvent) -> Option<&std::path::PathBuf> {
    match evt {
        DebouncedEvent::NoticeWrite(b)
        | DebouncedEvent::NoticeRemove(b)
        | DebouncedEvent::Create(b)
        | DebouncedEvent::Write(b)
        | DebouncedEvent::Chmod(b)
        | DebouncedEvent::Remove(b)
        | DebouncedEvent::Rename(b, _) => Some(b),
        DebouncedEvent::Error(_, _) | DebouncedEvent::Rescan => None,
    }
}

impl From<DebouncedEvent> for WatchEventType {
    fn from(e: DebouncedEvent) -> WatchEventType {
        match e {
            DebouncedEvent::Chmod(_) => WatchEventType::Touched,
            DebouncedEvent::Create(_) => WatchEventType::Touched,
            DebouncedEvent::Remove(_) => WatchEventType::Changed,
            DebouncedEvent::Rename(_, _) => WatchEventType::Changed,
            DebouncedEvent::Write(_) => WatchEventType::Changed,
            DebouncedEvent::NoticeRemove(_) => WatchEventType::Ignore,
            DebouncedEvent::NoticeWrite(_) => WatchEventType::Ignore,
            DebouncedEvent::Rescan => WatchEventType::Ignore,
            DebouncedEvent::Error(_, _) => WatchEventType::Ignore,
        }
    }
}

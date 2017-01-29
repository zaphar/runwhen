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

#[derive(PartialEq,Clone)]
pub enum WatchEventType {
    Touched,
    Changed,
    Error,
    Ignore
}

impl From<DebouncedEvent> for WatchEventType {
    fn from(e: DebouncedEvent) -> WatchEventType {
        println!("Found event: {:?}", e);
        match e {
            DebouncedEvent::Chmod(_) => WatchEventType::Touched,
            DebouncedEvent::Create(_) => WatchEventType::Touched,
            DebouncedEvent::Remove(_) => WatchEventType::Changed,
            DebouncedEvent::Rename(_, _) => WatchEventType::Changed,
            DebouncedEvent::Write(_) => WatchEventType::Changed,
            DebouncedEvent::NoticeRemove(_) => WatchEventType::Ignore,
            DebouncedEvent::NoticeWrite(_) => WatchEventType::Ignore,
            DebouncedEvent::Rescan => WatchEventType::Ignore,
            DebouncedEvent::Error(_, _) => WatchEventType::Ignore
        }
    }
}

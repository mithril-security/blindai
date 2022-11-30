// Copyright 2022 Mithril Security. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn find_split_index(data:&[u8],pattern:&[u8]) -> Option<usize> {

    if pattern == b"INPUT" {
        
        let pos = 
        data.windows(pattern.len())
            .enumerate()
            .find(|(_, w)| matches!(*w, b"INPUT"))
            .map(|(i, _)| i);
        return pos
    }
    
    if pattern == b"OUTPUT" {
        let pos = 
        data.windows(pattern.len())
            .enumerate()
            .find(|(_, w)| matches!(*w, b"OUTPUT"))
            .map(|(i, _)| i);
            
        return pos
    }
    return None  
    }
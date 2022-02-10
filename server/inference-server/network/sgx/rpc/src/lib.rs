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

use tonic_rpc::tonic_rpc;

// The `tonic_rpc` attribute says that we want to build an RPC defined by this trait.
// The `json` option says that we should use the `tokio-serde` Json codec for serialization.
#[tonic_rpc(json)]
pub trait UntrustedLocalApp {
    fn set_token(token:  String);
}

/*
 * Created on Wed May 05 2021
 *
 * Copyright (c) 2021 Sayan Nandan <nandansayan@outlook.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *    http://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

//! # Skytable client
//!
//! This library is the official client for the free and open-source NoSQL database
//! [Skytable](https://github.com/skytable/skytable). First, go ahead and install Skytable by
//! following the instructions [here](https://docs.skytable.io/getting-started). This library supports
//! all Skytable versions that work with the [Skyhash 1.0 Protocol](https://docs.skytable.io/protocol/skyhash).
//! This version of the library was tested with the latest Skytable release
//! (release [0.6](https://github.com/skytable/skytable/releases/v0.6.0)).
//!
//! ## Using this library
//!
//! This library only ships with the bare minimum that is required for interacting with Skytable. Once you have
//! Skytable installed and running, you're ready to follow this guide!
//!
//! We'll start by creating a new binary application and then running actions. Create a new binary application
//! by running:
//! ```shell
//! cargo new skyapp
//! ```
//! **Tip**: You can see a full list of the available actions [here](https://docs.skytable.io/actions-overview).
//!
//! First add this to your `Cargo.toml` file:
//! ```toml
//! skytable = "0.3.0"
//! ```
//! Now open up your `src/main.rs` file and establish a connection to the server while also adding some
//! imports:
//! ```no_run
//! use skytable::{Connection, Query, Response, Element};
//! fn main() -> std::io::Result<()> {
//!     let mut con = Connection::new("127.0.0.1", 2003)?;
//!     Ok(())
//! }
//! ```
//!
//! Now let's run a [`Query`]! Change the previous code block to:
//! ```no_run
//! use skytable::{Connection, Query, Response, Element};
//! fn main() -> std::io::Result<()> {
//!     let mut con = Connection::new("127.0.0.1", 2003)?;
//!     let query = Query::new("heya");
//!     let res = con.run_simple_query(&query)?;
//!     assert_eq!(res, Response::Item(Element::String("HEY!".to_owned())));
//!     Ok(())
//! }
//! ```
//!
//! Way to go &mdash; you're all set! Now go ahead and run more advanced queries!
//!
//! ## Async API
//! 
//! If you need to use an `async` API, just change your import to:
//! ```toml
//! skytable = { version = "0.3", features=["async"], default-features=false }
//! ```
//! You can now establish a connection by using `skytable::AsyncConnection::new()`, adding `.await`s wherever
//! necessary. Do note that you'll the [Tokio runtime](https://tokio.rs).
//! 
//! ## Using both `sync` and `async` APIs
//! 
//! With this client driver, it is possible to use both sync and `async` APIs **at the same time**. To do
//! this, simply change your import to:
//! ```toml
//! skytable = { version="0.3", features=["sync", "async"] }
//! ```
//!
//! ## Contributing
//!
//! Open-source, and contributions ... &mdash; they're always welcome! For ideas and suggestions,
//! [create an issue on GitHub](https://github.com/skytable/client-rust/issues/new) and for patches,
//! fork and open those pull requests [here](https://github.com/skytable/client-rust)!
//!
//! ## License
//! This client library is distributed under the permissive
//! [Apache-2.0 License](https://github.com/skytable/client-rust/blob/next/LICENSE). Now go build great apps!
//!

#![cfg_attr(docsrs, feature(doc_cfg))]
pub mod actions;
mod deserializer;
mod respcode;
pub mod types;

use std::io::Result as IoResult;
use types::IntoSkyhashAction;
use types::IntoSkyhashBytes;
// async imports
#[cfg(feature = "async")]
mod async_con;
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use async_con::Connection as AsyncConnection;
#[cfg(feature = "async")]
use tokio::io::AsyncWriteExt;
#[cfg(feature = "async")]
use tokio::net::TcpStream;
// default imports
pub use deserializer::Element;
pub use respcode::RespCode;
// sync imports
#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
mod sync;
#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub use sync::Connection;

#[macro_export]
/// A macro that can be used to easily create queries with _almost_ variadic properties.
/// Where you'd normally create queries like this:
/// ```
/// use skytable::Query;
/// let q = Query::new("mset").arg("x").arg("100").arg("y").arg("200");
/// ```
/// with this macro, you can just do this:
/// ```ignore
/// use skytable::query;
/// let q = query!("mset", "x", "100", "y", "200");
/// ```
macro_rules! query {
    ($($arg:expr),+) => {
        crate::Query::new_empty()$(.arg($arg))*
    };
}

#[derive(Debug, PartialEq)]
/// This struct represents a single simple query as defined by the Skyhash protocol
///
/// A simple query is serialized into a flat string array which is nothing but a Skyhash serialized equivalent
/// of an array of [`String`] items. To construct a query like `SET x 100`, one needs to:
/// ```
/// use skytable::Query;
/// fn main() {
///     let q = Query::new("set").arg("x").arg("100");
/// }
/// ```
/// You can now run this with a [`Connection`] or an `AsyncConnection`
///
pub struct Query {
    size_count: usize,
    data: Vec<u8>,
}

impl Query {
    /// Create a new query with an argument
    pub fn new(start: impl IntoSkyhashBytes) -> Self {
        Self::new_empty().arg(start)
    }
    /// Create a new empty query without any arguments
    pub fn new_empty() -> Self {
        Query {
            size_count: 0,
            data: Vec::new(),
        }
    }
    /// Add an argument to a query
    ///
    /// ## Panics
    /// This method will panic if the passed `arg` is empty
    pub fn arg(mut self, arg: impl IntoSkyhashAction) -> Self {
        arg.extend_bytes(&mut self.data);
        self.size_count += arg.incr_len_by();
        self
    }
    /// Number of items in the datagroup
    fn __len(&self) -> usize {
        self.size_count
    }
    fn get_holding_buffer(&self) -> &[u8] {
        &self.data
    }
    #[cfg(feature = "async")]
    /// Write a query to a given stream
    async fn write_query_to(&self, stream: &mut tokio::io::BufWriter<TcpStream>) -> IoResult<()> {
        // Write the metaframe
        stream.write_all(b"*1\n").await?;
        // Add the dataframe
        let number_of_items_in_datagroup = self.__len().to_string().into_bytes();
        stream.write_all(&[b'_']).await?;
        stream.write_all(&number_of_items_in_datagroup).await?;
        stream.write_all(&[b'\n']).await?;
        stream.write_all(self.get_holding_buffer()).await?;
        Ok(())
    }
    #[cfg(feature = "sync")]
    /// Write a query to a given stream
    fn write_query_to_sync(&self, stream: &mut std::net::TcpStream) -> IoResult<()> {
        use std::io::Write;
        // Write the metaframe
        stream.write_all(b"*1\n")?;
        // Add the dataframe
        let number_of_items_in_datagroup = self.__len().to_string().into_bytes();
        stream.write_all(&[b'_'])?;
        stream.write_all(&number_of_items_in_datagroup)?;
        stream.write_all(&[b'\n'])?;
        stream.write_all(self.get_holding_buffer())?;
        Ok(())
    }
    #[cfg(feature = "dbg")]
    #[cfg_attr(docsrs, doc(cfg(feature = "dbg")))]
    /// Get the raw bytes of a query
    ///
    /// This is a function that is **not intended for daily use** but is for developers working to improve/debug
    /// or extend the Skyhash protocol. [Skytable](https://github.com/skytable/skytable) itself uses this function
    /// to generate raw queries. Once you're done passing the arguments to a query, running this function will
    /// return the raw query that would be written to the stream, serialized using the Skyhash serialization protocol
    pub fn into_raw_query(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.data.len());
        v.extend(b"*1\n");
        v.extend(b"_");
        v.extend(self.__len().to_string().into_bytes());
        v.extend(b"\n");
        v.extend(self.get_holding_buffer());
        v
    }
}

/// # Responses
///
/// This enum represents responses returned by the server. This can either be an array (or bulk), a single item
/// or can be a parse error if the server returned some data but it couldn't be parsed into the expected type
/// or it can be an invalid response in the event the server sent some invalid data.
/// This enum is `#[non_exhaustive]` as more types of responses can be added in the future.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Response {
    /// The server sent an invalid response
    InvalidResponse,
    /// The server responded with _something_. This can be any of the [`Element`] variants
    Item(Element),
    /// We failed to parse data
    ParseError,
    /// The server sent some data of a type that this client doesn't support
    UnsupportedDataType,
}

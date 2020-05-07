#![recursion_limit = "512"]

#[macro_use]
extern crate serde;

pub mod channel;
pub mod cmd;
pub mod proto_gen;
pub mod route;

use crate::cmd::PeerCmd;
use route::*;
use std::sync::{Arc, Mutex};
use warp::filters::path::Peek;
use warp::reject;
use warp::Filter;

#[tokio::main]
async fn main() {
    let peer_cmd = PeerCmd::new("mychannel"); // you can change the channel ID if needed

    let shared = Arc::new(Mutex::new(peer_cmd));
    let shared = warp::any().map(move || Arc::clone(&shared));

    let block = warp::any()
        .and(warp::path("block"))
        .and(warp::path::peek())
        .and_then(|peek: Peek| async move {
            peek.as_str()
                .parse::<u64>()
                .map_err(|_| reject::not_found())
        })
        .and(shared.clone())
        .and_then(block);

    let blocks = warp::any()
        .and(warp::path("blocks"))
        .and(shared.clone())
        .and_then(blocks);

    let styles = warp::get().and(warp::path("styles.css")).map(|| {
        r#"
        .block {
            width: 568px;
            min-height: 36px;
            border: 2px solid black;
            background: ghostwhite;
            padding-left: 15px;
            margin-top: 5px;
            border-radius: 4px;
        }
        .block-full {
            width: fit-content;
            border: 2px solid black;
            background: ghostwhite;
            padding-left: 15px;
            padding-right: 15px;
            padding-bottom: 15px;
            margin-top: 17px;
            border-radius: 6px;
            box-shadow: 0 3px 6px rgba(0,0,0,0.16), 0 3px 6px rgba(0,0,0,0.23);
        }
        "#
    });
    let routes = block.or(blocks).or(styles);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

use crate::cmd::PeerCmd;
use mime::Mime;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use typed_html::dom::DOMTree;
use typed_html::{html, text};
use warp::reply;

pub async fn blocks(shared: Arc<Mutex<PeerCmd>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut shared = shared.lock().unwrap();
    let channel_info = shared.get_channel_info().ok_or_else(|| warp::reject())?;
    let last_height = channel_info.height;
    let blocks: Vec<_> = (0..last_height)
        .filter_map(|h| shared.fetch_and_get_block(h))
        .collect();
    use typed_html::types::LinkType;
    let css = Mime::from_str("text/css").unwrap();

    let doc: DOMTree<String> = html!(
        <html>
            <head>
                <title>"Blocks info"</title>
                <link type=css rel=LinkType::StyleSheet href="/styles.css"></link>
            </head>
            <body>
                <h3>{text!("Blocks")}</h3>
                { blocks.into_iter().map(|b| html!(
                    <div class="block">
                        <p> { text!("Height: {}", b.height) } </p>
                        <p> { text!("Hash: ") } <a href={format!("/block/{}", b.height)}> { text!("{}", b.hash) } </a> </p>
                    </div>
                )) }
            </body>
        </html>
    );
    Ok(reply::html(doc.to_string()))
}

pub async fn block(
    height: u64,
    shared: Arc<Mutex<PeerCmd>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut shared = shared.lock().unwrap();
    let block_info = shared
        .fetch_and_get_block(height)
        .ok_or_else(|| warp::reject())?;
    use typed_html::types::LinkType;
    let css = mime::Mime::from_str("text/css").unwrap();

    let doc: DOMTree<String> = html!(
        <html>
            <head>
                <title>"Block info"</title>
                <link type=css rel=LinkType::StyleSheet href="/styles.css"></link>
            </head>
            <body>
                <h3><a href="/blocks">"Back"</a></h3>
                <h2>{text!("Block #{} info", height)}</h2>
                <div class="block-full">
                    <p> { text!("Height: {}", block_info.height) } </p>
                    <p> { text!("Hash: {}", block_info.hash) } </p>
                    {
                        block_info.transactions.into_iter().map(|tx| html!(
                            <div class="block">
                                <p><b> { text!("ID: {}", tx.tx_id) } </b></p>
                                <p> { text!("Is valid: {}", tx.valid) } </p>
                                <p> { text!("Timestamp: {}", tx.timestamp) } </p>
                                <p> { text!("MSP ID: {}", tx.mspid) } </p>
                            </div>
                        ))
                    }
                </div>
            </body>
        </html>
    );
    Ok(reply::html(doc.to_string()))
}

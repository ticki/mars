extern crate mars;

use mars::{Bot, Response};

fn main() {
    Bot::new("TOKENHERE", |_| Response {
        username: Some("pong-bot"),
        text: "pong".into(),
        icon_url: Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg"),
    }).init("127.0.0.1:80").unwrap();
}

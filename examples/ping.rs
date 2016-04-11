extern crate mars;

use mars::{Bot, Response};

fn main() {
    Bot::new(|_| Response {
        username: Some("pong-bot"),
        text: "pong".into(),
        icon_url: Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg"),
    }).init().unwrap();
}

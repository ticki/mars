#![feature(str_escape)]

extern crate hyper;
extern crate url;

use url::form_urlencoded as post;

use std::io::{self, Write, Read};

/// The trigger word ro command.
pub enum Trigger {
    /// A slash command.
    Command(String),
    /// A trigger word.
    Word(String),
}

impl Default for Trigger {
    fn default() -> Trigger {
        Trigger::Word(String::new())
    }
}

/// A Mattermost request.
///
/// These are often sent by the Mattermost server, due to being triggered by a slash command or
/// a keyword. To see how to configure it, see the [Mattermost
/// docs](http://docs.mattermost.com/developer/webhooks-outgoing.html).
#[derive(Default)]
pub struct Request {
    /// The alphanumeric channel identifier.
    pub channel_id: String,
    /// The name of the channel.
    pub channel_name: String,
    /// The domain name of the team.
    pub team_domain: String,
    /// The alphanumeric team identifier.
    pub team_id: String,
    /// The text message payload.
    pub text: String,
    /// The timestamp.
    pub timestamp: String,
    /// The API token.
    pub token: String,
    /// The trigger of this request.
    pub trigger: Trigger,
    /// The trigger user's alphanumeric identifier.
    pub user_id: String,
    /// The trigger user's username.
    pub username: String,
}

impl Request {
    fn from_bytes(b: &[u8]) -> Request {
        let mut req = Request::default();
        let query = post::parse(b);

        for (a, b) in query {
            match a.as_str() {
                "channel_id" => req.channel_id = b,
                "channel_name" => req.channel_name = b,
                "team_domain" => req.team_domain = b,
                "team_id" => req.team_id = b,
                "text" => req.text = b,
                "timestamp" => req.timestamp = b,
                "token" => req.token = b,
                "trigger_word" => req.trigger = Trigger::Word(b),
                "command" => req.trigger = Trigger::Command(b),
                "user_id" => req.user_id = b,
                "user_name" => req.username = b,
                _ => (),
            }
        }

        req
    }
}

/// The response to the request.
pub struct Response<'a> {
    /// The bot's username.
    pub username: Option<&'a str>,
    /// The payload text.
    pub text: String,
    /// The URL to the bot's avatar.
    pub icon_url: Option<&'a str>,
}

impl<'a> Response<'a> {
    fn send<W: Write>(self, mut to: W) -> io::Result<()> {
        try!(to.write_all(b"{\"text\": \""));
        try!(to.write_all(self.text.escape_default().as_bytes()));
        try!(to.write_all(b"\""));
        if let Some(x) = self.username {
            try!(to.write_all(b", \"username\": \""));
            try!(to.write_all(x.escape_default().as_bytes()));
            try!(to.write_all(b"\""));
        }
        if let Some(x) = self.icon_url {
            try!(to.write_all(b", \"icon_url\": \""));
            try!(to.write_all(x.escape_default().as_bytes()));
            try!(to.write_all(b"\""));
        }
        try!(to.write_all(b"}"));

        Ok(())
    }
}

/// A Mattermost bot.
pub struct Bot<F> {
    handler: F,
    token: &'static str,
}

impl<'a, F> Bot<F> where F: 'static + Sync + Send + Fn(Request) -> Response<'a> {
    /// Create a new bot with a given handler.
    pub fn new(token: &'static str, handler: F) -> Bot<F> {
        Bot {
            handler: handler,
            token: token,
        }
    }

    /// Initialize the bot.
    pub fn init(self, ip: &str) -> Result<(), hyper::Error> {
        try!(try!(hyper::Server::http(ip)).handle(move |mut req: hyper::server::Request, mut res: hyper::server::Response| {
            let mut vec = Vec::new();
            if req.read_to_end(&mut vec).is_err() {
                *res.status_mut() = hyper::BadRequest;
                return;
            }

            let trig = Request::from_bytes(&vec);
            if &trig.token != self.token {
                // Token mismatch.
                *res.status_mut() = hyper::status::StatusCode::Unauthorized;
                return;
            }


            if let Ok(res) = res.start() {
                let _ = (self.handler)(trig).send(res);
            }

        }));

        Ok(())
    }
}

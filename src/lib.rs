#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate extra;
extern crate hyper;
extern crate url;
extern crate serde;
extern crate serde_json as json;

use url::form_urlencoded as post;

use extra::option::OptionalExt;

use std::borrow::Cow;
use std::io::{self, Read, Write};

/// A Mattermost request.
///
/// These are often sent by the Mattermost server, due to being triggered by a slash command or
/// a keyword. To see how to configure it, see the [Mattermost
/// docs](http://docs.mattermost.com/developer/webhooks-outgoing.html).
pub struct Request<'a> {
    /// The alphanumeric channel identifier.
    pub channel_id: Cow<'a, str>,
    /// The name of the channel.
    pub channel_name: Cow<'a, str>,
    /// The domain name of the team.
    pub team_domain: Cow<'a, str>,
    /// The alphanumeric team identifier.
    pub team_id: Cow<'a, str>,
    /// The text message payload.
    pub text: Cow<'a, str>,
    /// The timestamp.
    pub timestamp: Cow<'a, str>,
    /// The API token.
    pub token: Cow<'a, str>,
    /// The trigger of this request.
    pub trigger: Cow<'a, str>,
    /// The trigger user's alphanumeric identifier.
    pub user_id: Cow<'a, str>,
    /// The trigger user's username.
    pub username: Cow<'a, str>,
}

impl<'a> Default for Request<'a> {
    fn default() -> Request<'a> {
        Request {
            channel_id: Cow::Borrowed(""),
            channel_name: Cow::Borrowed(""),
            team_domain: Cow::Borrowed(""),
            team_id: Cow::Borrowed(""),
            text: Cow::Borrowed(""),
            timestamp: Cow::Borrowed(""),
            token: Cow::Borrowed(""),
            trigger: Cow::Borrowed(""),
            user_id: Cow::Borrowed(""),
            username: Cow::Borrowed(""),
        }
    }
}

impl<'a> Request<'a> {
    fn from_bytes(b: &[u8]) -> Request {
        let mut req = Request::default();
        let query = post::parse(b);

        for (a, b) in query {
            match &*a {
                "channel_id" => req.channel_id = b,
                "channel_name" => req.channel_name = b,
                "team_domain" => req.team_domain = b,
                "team_id" => req.team_id = b,
                "text" => req.text = b,
                "timestamp" => req.timestamp = b,
                "token" => req.token = b,
                "trigger_word" => req.trigger = b,
                "user_id" => req.user_id = b,
                "user_name" => req.username = b,
                _ => (),
            }
        }

        req
    }
}

/// The response to the request.
#[derive(Serialize)]
pub struct Response<'a> {
    /// The bot's username.
    #[serde(skip_serializing_if="Option::is_none")]
    pub username: Option<Cow<'a, str>,>,
    /// The payload text.
    pub text: Cow<'a, str>,
    /// The URL to the bot's avatar.
    #[serde(skip_serializing_if="Option::is_none")]
    pub icon_url: Option<Cow<'a, str>>,
}

impl<'a> Response<'a> {
    fn send<W: Write>(self, mut to: W) -> io::Result<()> {
        match json::to_writer(&mut to, &self) {
            Ok(()) => Ok(()),
            Err(json::Error::Io(x)) => Err(x),
            Err(x) => Err(io::Error::new(io::ErrorKind::InvalidData, x)),
        }
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
            let mut stderr = io::stderr();
            let mut vec = Vec::new();

            if let None = req.read_to_end(&mut vec).warn(&mut stderr) {
                *res.status_mut() = hyper::BadRequest;
                return;
            }

            let trig = Request::from_bytes(&vec);
            if &trig.token != self.token {
                let _ = write!(stderr, "warning: token mismatch.");

                // Token mismatch.
                *res.status_mut() = hyper::status::StatusCode::Unauthorized;
                return;
            }


            if let Some(res) = res.start().warn(&mut stderr) {
                (self.handler)(trig).send(res).warn(&mut stderr);
            }
        }));

        Ok(())
    }
}

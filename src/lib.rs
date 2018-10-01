//! # pixiv
//!
//! The `pixiv` crate provides an unofficial library for the Pixiv API.
//!
//! This crate uses the crates `reqwest` and `serde_json`.
//!
//! ## Authentication
//!
//! To authenticate, you need to create a new `Pixiv` struct and pass in a `reqwest::Client`, then login with your username and password.
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # use pixiv::Pixiv;
//! # use reqwest::Client;
//! # fn main() {
//!     let client = Client::new();
//!
//!     let mut pixiv: Pixiv = Pixiv::new(&client);
//!     pixiv.login("username", "password");
//! # }
//! ```
//!
//! If and when your access token does expire, you should use the `refresh_auth()` or `login()` methods.
//!
//! Alternatively, if you have your access token and/or request token cached somwhere for you to reuse:
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # use pixiv::Pixiv;
//! # use reqwest::Client;
//! # fn main() {
//!     let client = Client::new();
//!
//!     let mut pixiv: Pixiv = Pixiv::new(&client);
//!
//!     let my_access_token = String::from("supersecret");
//!     *pixiv.access_token_mut() = my_access_token;
//! # }
//! ```
//!
//! Accessor methods such as `access_token()` and `refresh_token()` are provided for these purposes.
//!
//! ## Making a Request
//!
//! This crate relies on the builder pattern for using and modifying a request. A typical request may look like this:
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # extern crate serde_json;
//! # use pixiv::Pixiv;
//! # use reqwest::Client;
//! # use serde_json::Value;
//! # fn main() {
//! #   let client = Client::new();
//! #   let mut pixiv: Pixiv = Pixiv::new(&client);
//! #   pixiv.login("username", "password");
//!     let work: Value = pixiv
//!         .work(66024340)
//!         .send()
//!         .expect("Request failed.")
//!         .json()
//!         .expect("Failed to parse as json.");
//! # }
//! ```
//!
//! A more complicated response may look like this:
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # extern crate serde_json;
//! # use pixiv::Pixiv;
//! # use reqwest::Client;
//! # use serde_json::Value;
//! # fn main() {
//! #   let client = Client::new();
//! #   let mut pixiv: Pixiv = Pixiv::new(&client);
//! #   pixiv.login("username", "password");
//!     let following_works: Value = pixiv
//!        .following_works()
//!        .image_sizes(&["large"])
//!        .include_sanity_level(false)
//!        .send()
//!        .expect("Request failed.")
//!        .json()
//!        .expect("Failed to parse as json.");
//! # }
//! ```
//!
//! Since `work` is of type `serde_json::Value`, it's up to you to figure out how you want to parse this response for your program.
//!
//! You may want to refer [here](https://www.snip2code.com/Snippet/798193/Unofficial-API-specification-extracted-f) for what a response from Pixiv may look like.
//!
//! ## Future Support (Maybe)
//!
//! * More examples!
//! * More versatile support for handling and parsing responses (instead of just the raw response)
//! * More API support (although pixiv doesn't document their public API anywhere to my knowledge...)

extern crate chrono;
pub extern crate reqwest;
pub extern crate http;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
extern crate kankyo;

#[macro_use]
extern crate log;

use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Write};

use chrono::naive::NaiveDate;

use reqwest::Client;
use reqwest::Response;
use http::status::StatusCode;
pub use http::{header, HeaderMap, Method};
use reqwest::Url;

use serde_json::Value;

// This is taken from the Android app, don't worry about it. It's not really "compromisable", to some degree.
const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

/// Used to authenticate to the Pixiv servers and construct Pixiv requests through methods creating `PixivRequestBuilder`.
#[derive(Debug, Clone)]
pub struct Pixiv {
    client: Client,
    access_token: String,
    refresh_token: String,
}

/// Pixiv request. You can create this using `PixivRequestBuilder::build`. This is for if you wish to inspect the request before sending.
#[derive(Debug, Clone)]
pub struct PixivRequest {
    method: Method,
    url: Url,
    headers: HeaderMap,
}

/// Pixiv request builder. You can create this using any of the provided methods in `Pixiv`, or through `PixivRequestBuilder::new`.
#[derive(Debug, Clone)]
pub struct PixivRequestBuilder<'a> {
    pixiv: &'a Pixiv,
    request: PixivRequest,
    params: HashMap<&'a str, Cow<'a, str>>,
}
/// Error returned on failure to authorize with pixiv.
#[derive(Debug)]
pub struct AuthError {
    reason: String,
}

impl Error for AuthError {
    fn description(&self) -> &str {
        "An error occurred while trying to authenticate."
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "An error occurred while trying to authenticate. Reason: {:?}",
            self.reason
        )
    }
}

/// Enum to set publicity param.
#[derive(Debug, Clone, Copy)]
pub enum Publicity {
    Public,
    Private,
}

impl Publicity {
    fn as_str(&self) -> &'static str {
        match *self {
            Publicity::Public => "public",
            Publicity::Private => "private",
        }
    }
}

/// Enum to set ranking type param.
#[derive(Debug, Clone, Copy)]
pub enum RankingType {
    All,
    Illust,
    Manga,
    Ugoira,
}

impl RankingType {
    fn as_str(&self) -> &'static str {
        match *self {
            RankingType::All => "all",
            RankingType::Illust => "illust",
            RankingType::Manga => "manga",
            RankingType::Ugoira => "ugoira",
        }
    }
}

/// Enum to set ranking mode param.
#[derive(Debug, Clone, Copy)]
pub enum RankingMode {
    Daily,
    Weekly,
    Monthly,
    Rookie,
    Original,
    Male,
    Female,
    DailyR18,
    WeeklyR18,
    MaleR18,
    FemaleR18,
    R18G,
}

impl RankingMode {
    fn as_str(&self) -> &'static str {
        match *self {
            RankingMode::Daily => "daily",
            RankingMode::Weekly => "weekly",
            RankingMode::Monthly => "monthly",
            RankingMode::Rookie => "rookie",
            RankingMode::Original => "original",
            RankingMode::Male => "male",
            RankingMode::Female => "female",
            RankingMode::DailyR18 => "daily_r18",
            RankingMode::WeeklyR18 => "weekly_r18",
            RankingMode::MaleR18 => "male_r18",
            RankingMode::FemaleR18 => "female_r18",
            RankingMode::R18G => "r18g",
        }
    }
}

/// Enum to set search period param.
#[derive(Debug, Clone, Copy)]
pub enum SearchPeriod {
    All,
    Day,
    Week,
    Month,
}

impl SearchPeriod {
    fn as_str(&self) -> &'static str {
        match *self {
            SearchPeriod::All => "all",
            SearchPeriod::Day => "day",
            SearchPeriod::Week => "week",
            SearchPeriod::Month => "month",
        }
    }
}

/// Enum to set search mode param.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    Text,
    Tag,
    ExactTag,
    Caption,
}

impl SearchMode {
    fn as_str(&self) -> &'static str {
        match *self {
            SearchMode::Text => "text",
            SearchMode::Tag => "tag",
            SearchMode::ExactTag => "exact_tag",
            SearchMode::Caption => "caption",
        }
    }
}

/// Enum to set search order param.
#[derive(Debug, Clone, Copy)]
pub enum SearchOrder {
    Descending,
    Ascending,
}

impl SearchOrder {
    fn as_str(&self) -> &'static str {
        match *self {
            SearchOrder::Descending => "desc",
            SearchOrder::Ascending => "asc",
        }
    }
}

impl Pixiv {
    /// Creates a new Pixiv struct.
    #[inline]
    pub fn new(client: &Client) -> Pixiv {
        Pixiv {
            client: client.clone(),
            access_token: String::default(),
            refresh_token: String::default(),
        }
    }
    /// This is required to use all the other functions this library provides. Requires a valid username and password.
    pub fn login(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
        let mut data = HashMap::new();

        data.insert("client_id", CLIENT_ID);
        data.insert("client_secret", CLIENT_SECRET);
        data.insert("get_secure_url", "1");

        data.insert("grant_type", "password");
        data.insert("username", username);
        data.insert("password", password);

        let mut res = self.send_auth_request(&data)
            .expect("Error occured while requesting token.");

        match res.status() {
            StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => {
                // success
            }
            s => {
                return Err(AuthError {
                    reason: format!(
                        "Login failed. Check your username and password. Response: {:?}",
                        s
                    ),
                })
            }
        }

        let mut json_response: Value = res.json().unwrap();

        self.access_token = match json_response["response"]["access_token"].take() {
            Value::String(s) => s,
            _ => panic!("Failed to get access token."),
        };
        self.refresh_token = match json_response["response"]["refresh_token"].take() {
            Value::String(s) => s,
            _ => panic!("Failed to get refresh token."),
        };
        Ok(())
    }
    /// Refreshes the authentication. You should use this when your access token is close to expiring.
    pub fn refresh_auth(&mut self) -> Result<(), AuthError> {
        let refresh_clone = self.refresh_token.clone();
        let mut data = HashMap::new();

        data.insert("client_id", CLIENT_ID);
        data.insert("client_secret", CLIENT_SECRET);
        data.insert("get_secure_url", "1");

        data.insert("grant_type", "refresh_token");
        data.insert("refresh_token", refresh_clone.as_str());

        let mut res = self.send_auth_request(&data)
            .expect("Error occured while requesting token.");

        match res.status() {
            StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => {
                // success
            }
            s => {
                return Err(AuthError {
                    reason: format!("Login failed. Check your refresh token. Response: {:?}", s),
                })
            }
        }

        let mut json_response: Value = res.json().unwrap();

        self.access_token = match json_response["response"]["access_token"].take() {
            Value::String(s) => s,
            _ => panic!("Failed to get access token."),
        };
        self.refresh_token = match json_response["response"]["refresh_token"].take() {
            Value::String(s) => s,
            _ => panic!("Failed to get refresh token."),
        };
        Ok(())
    }
    /// Get the access token.
    #[inline]
    pub fn access_token(&self) -> &String {
        &self.access_token
    }
    /// Get a mutable reference to the access token.
    #[inline]
    pub fn access_token_mut(&mut self) -> &mut String {
        &mut self.access_token
    }
    /// Get the refresh token.
    #[inline]
    pub fn refresh_token(&self) -> &String {
        &self.refresh_token
    }
    /// Get a mutable reference to the refresh token.
    #[inline]
    pub fn refresh_token_mut(&mut self) -> &mut String {
        &mut self.refresh_token
    }
    // private helper method
    fn send_auth_request(&self, data: &HashMap<&str, &str>) -> Result<Response, reqwest::Error> {
        self.client
            .post("https://oauth.secure.pixiv.net/auth/token")
            .form(&data)
            .send()
    }
    /// Used to build a request to retrive `bad_words.json`.
    /// # Request Transforms
    /// None
    pub fn bad_words(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1.1/bad_words.json";
        let url = Url::parse(&url).unwrap();
        PixivRequestBuilder::new(
            self,
            Method::GET,
            url,
            HashMap::default(),
        )
    }
    /// Used to build a request to retrieve information of a work.
    /// # Request Transforms
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    pub fn work(&self, illust_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/works/{}.json",
            illust_id
        );
        let extra_params = [
            ("image_sizes", "px_128x128,small,medium,large,px_480mw"),
            ("include_stats", "true"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve information of a user.
    /// # Request Transforms
    /// * `profile_image_sizes` (default: `px_170x170,px_50x50`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    pub fn user(&self, user_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}.json",
            user_id
        );
        let extra_params = [
            ("profile_image_sizes", "px_170x170,px_50x50"),
            ("image_sizes", "px_128x128,small,medium,large,px_480mw"),
            ("include_stats", "1"),
            ("include_profile", "1"),
            ("include_workspace", "1"),
            ("include_contacts", "1"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve your account's feed.
    /// # Request Transforms
    /// * `show_r18` (default: `true`)
    /// * `max_id`
    pub fn feed(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/feeds.json";
        let extra_params = [
            ("relation", "all"),
            ("type", "touch_nottext"),
            ("show_r18", "1"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve works favorited on your account.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `50`)
    /// * `publicity` (default: `public`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    pub fn favorite_works(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let extra_params = [
            ("page", "1"),
            ("per_page", "50"),
            ("publicity", "public"),
            ("image_sizes", "px_128x128,px_480mw,large"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to favorite a work on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_work_add(&self, work_id: usize) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("work_id", work_id.to_string().into())))
            .collect();
        PixivRequestBuilder::new(
            self,
            Method::POST,
            url,
            params,
        )
    }
    /// Used to build a request to remove favorited works on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_works_remove<B, I>(&self, work_ids: I) -> PixivRequestBuilder
    where
        B: Borrow<usize>,
        I: IntoIterator<Item = B>,
    {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("ids", comma_delimited(work_ids).into())))
            .collect();
        PixivRequestBuilder::new(
            self,
            Method::DELETE,
            url,
            params,
        )
    }
    /// Used to build a request to retrieve newest works from whoever you follow on your account.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn following_works(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/following/works.json";
        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve users you follow.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn following(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/following.json";
        let extra_params = [("page", "1"), ("per_page", "30"), ("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to follow a user on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_add(&self, user_id: usize) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("target_user_id", user_id.to_string().into())))
            .collect();
        PixivRequestBuilder::new(
            self,
            Method::POST,
            url,
            params,
        )
    }
    /// Used to build a request to unfollow users on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_remove<B, I>(&self, user_ids: I) -> PixivRequestBuilder
    where
        B: Borrow<usize>,
        I: IntoIterator<Item = B>,
    {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("delete_ids", comma_delimited(user_ids).into())))
            .collect();
        PixivRequestBuilder::new(
            self,
            Method::DELETE,
            url,
            params,
        )
    }
    /// Used to build a request to retrive a list of works submitted by a user.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn user_works(&self, user_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/works.json",
            user_id
        );
        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrive a list of works favorited by a user.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_sanity_level` (default: `true`)
    pub fn user_favorite_works(&self, user_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/favorite_works.json",
            user_id
        );
        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("include_sanity_level", "true"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrive a user's feed.
    /// # Request Transforms
    /// * `show_r18` (default: `true`)
    pub fn user_feed(&self, user_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/feeds.json",
            user_id
        );
        let extra_params = [
            ("relation", "all"),
            ("type", "touch_nottext"),
            ("show_r18", "1"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve users a user follows.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `max_id`
    pub fn user_following(&self, user_id: usize) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/following.json",
            user_id
        );
        let extra_params = [("page", "1"), ("per_page", "30")];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve a list of ranking posts.
    /// # Request Transforms
    /// * `ranking_mode` (default: `RankingMode::Daily`)
    /// * `page` (default: `1`)
    /// * `per_page` (default: `50`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `profile_image_sizes` (default: `px_170x170,px_50x50`)
    pub fn ranking(&self, ranking_type: RankingType) -> PixivRequestBuilder {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/ranking/{}.json",
            ranking_type.as_str()
        );
        let extra_params = [
            ("mode", "daily"),
            ("page", "1"),
            ("per_page", "50"),
            ("include_stats", "True"),
            ("include_sanity_level", "True"),
            ("image_sizes", "px_128x128,small,medium,large,px_480mw"),
            ("profile_image_sizes", "px_170x170,px_50x50"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to search for posts on a query.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `date`
    /// * `search_mode` (default: `SearchMode::Text`)
    /// * `search_period` (default: `SearchPeriod::All`)
    /// * `search_order` (default: `desc`)
    /// * `search_sort` (default: `date`)
    /// * `search_types` (default: `illustration,manga,ugoira`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    pub fn search_works<'a, V>(&'a self, query: V) -> PixivRequestBuilder<'a>
    where
        Cow<'a, str>: From<V>,
    {
        let url = "https://public-api.secure.pixiv.net/v1/search/works.json";
        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("mode", "text"),
            ("period", "all"),
            ("order", "desc"),
            ("sort", "date"),
            ("types", "illustration,manga,ugoira"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
            ("image_sizes", "px_128x128,px480mw,large"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("q", query.into())))
            .collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Used to build a request to retrieve the latest submitted works by everyone.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `50`)
    /// * `date`
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `profile_image_sizes` (default: `px_170x170,px_50x50`)
    pub fn latest_works(&self) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/works.json";
        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("profile_image_sizes", "px_170x170,px_50x50"),
        ];
        let url = Url::parse(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(self, Method::GET, url, params)
    }
    /// Executes a given `PixivRequest`.
    pub fn execute(&self, request: PixivRequest) -> Result<Response, reqwest::Error> {
        self.client.request(request.method,  request.url)
                   .headers(request.headers)
                   .bearer_auth(self.access_token.clone())
                   .send()
    }
}

impl PixivRequest {
    /// Create a new `PixivRequest`.
    /// A `PixivRequest` is returned when calling `build()` on `PixivRequestBuilder`, so it is recommended you use that instead.
    #[inline]
    pub fn new(method: Method, url: Url, headers: HeaderMap) -> PixivRequest {
        PixivRequest {
            method,
            url,
            headers,
        }
    }
    /// Get the method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }
    /// Get a mutable reference to the method.
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }
    /// Get the url.
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }
    /// Get a mutable reference to the url.
    #[inline]
    pub fn url_mut(&mut self) -> &Url {
        &mut self.url
    }
    /// Get the headers.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    fn extend_query_pairs<I, K, V>(&mut self, params: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.url.query_pairs_mut().extend_pairs(params);
    }
}

impl<'a> PixivRequestBuilder<'a> {
    /// Create a new `PixivRequestBuilder`.
    /// Functions in `Pixiv` expedite a lot of this for you, so using this directly isn't recommended unless you know what you want.
    pub fn new(
        pixiv: &'a Pixiv,
        method: Method,
        url: Url,
        params: HashMap<&'a str, Cow<'a, str>>,
    ) -> PixivRequestBuilder<'a> {
        // set headers
        let mut headers = HeaderMap::new();
        headers.insert(header::REFERER, header::HeaderValue::from_static("http://spapi.pixiv.net/"));

        PixivRequestBuilder {
            pixiv,
            request: PixivRequest::new(method, url, headers),
            params,
        }
    }
    /// Sets the `page` param.
    #[inline]
    pub fn page(self, value: usize) -> PixivRequestBuilder<'a> {
        self.raw_param("page", value.to_string())
    }
    /// Sets the `per_page` param.
    #[inline]
    pub fn per_page(self, value: usize) -> PixivRequestBuilder<'a> {
        self.raw_param("value", value.to_string())
    }
    /// Sets the `max_id` param.
    #[inline]
    pub fn max_id(self, value: usize) -> PixivRequestBuilder<'a> {
        self.raw_param("max_id", value.to_string())
    }
    /// Sets the `image_sizes` param. Available types: `px_128x128`, `small`, `medium`, `large`, `px_480mw`
    #[inline]
    pub fn image_sizes(self, values: &[&str]) -> PixivRequestBuilder<'a> {
        self.raw_param("image_sizes", comma_delimited::<&str, _, _>(values))
    }
    /// Sets the `profile_image_sizes` param. Available types: `px_170x170,px_50x50`
    #[inline]
    pub fn profile_image_sizes(self, values: &[&str]) -> PixivRequestBuilder<'a> {
        self.raw_param("profile_image_sizes", comma_delimited::<&str, _, _>(values))
    }
    /// Sets the `publicity` param. Must be a value of enum `Publicity`.
    #[inline]
    pub fn publicity(self, value: Publicity) -> PixivRequestBuilder<'a> {
        self.raw_param("publicity", value.as_str())
    }
    /// Sets the `show_r18` param. `true` means R-18 works will be included.
    #[inline]
    pub fn show_r18(self, value: bool) -> PixivRequestBuilder<'a> {
        if value {
            self.raw_param("show_r18", "1")
        } else {
            self.raw_param("show_r18", "0")
        }
    }
    /// Sets the `include_stats` param.
    #[inline]
    pub fn include_stats(self, value: bool) -> PixivRequestBuilder<'a> {
        if value {
            self.raw_param("include_stats", "true")
        } else {
            self.raw_param("include_stats", "false")
        }
    }
    /// Sets the `include_sanity_level` param.
    #[inline]
    pub fn include_sanity_level(self, value: bool) -> PixivRequestBuilder<'a> {
        if value {
            self.raw_param("include_sanity_level", "true")
        } else {
            self.raw_param("include_sanity_level", "false")
        }
    }
    /// Sets the ranking mode in the case of a `ranking()` call. Must be a value of enum `RankingMode`.
    #[inline]
    pub fn ranking_mode(self, value: RankingMode) -> PixivRequestBuilder<'a> {
        self.raw_param("mode", value.as_str())
    }
    /// Sets the `date` param. Must be a valid date in the form of `%Y-%m-%d`, e.g. `2018-2-22`.
    pub fn date<V>(self, value: V) -> PixivRequestBuilder<'a>
    where
        Cow<'a, str>: From<V>,
    {
        let value: Cow<_> = value.into();
        // just to validate the date format
        NaiveDate::parse_from_str(&*value, "%Y-%m-%d").expect("Invalid date or format given.");
        self.raw_param::<Cow<_>>("date", value)
    }
    /// Sets the `period` param in the case of a `search_works()` call. Must be a value of enum `SearchPeriod`.
    #[inline]
    pub fn search_period(self, value: SearchPeriod) -> PixivRequestBuilder<'a> {
        self.raw_param("period", value.as_str())
    }
    /// Sets the `mode` param in the case of a `search_works()` call. Must be a value of enum `SearchMode`.
    #[inline]
    pub fn search_mode(self, value: SearchMode) -> PixivRequestBuilder<'a> {
        self.raw_param("mode", value.as_str())
    }
    /// Sets the `order` param in the case of a `search_works()` call. Must be a value of enum `SearchOrder`.
    #[inline]
    pub fn search_order(self, value: SearchOrder) -> PixivRequestBuilder<'a> {
        self.raw_param("order", value.as_str())
    }
    /// Sets the `sort` param in the case of a `search_works()` call. Not sure if there's any variations here, but this function is included for convenience.
    pub fn search_sort<V>(self, value: V) -> PixivRequestBuilder<'a>
    where
        Cow<'a, str>: From<V>,
    {
        self.raw_param("sort", value)
    }
    /// Sets the `types` param in the case of a `search_works()` call. Available values: `illustration`, `manga`, `ugoira`.
    #[inline]
    pub fn search_types(self, values: &[&str]) -> PixivRequestBuilder<'a> {
        self.raw_param("types", comma_delimited::<&str, _, _>(values))
    }
    fn raw_param<V>(mut self, key: &'a str, value: V) -> PixivRequestBuilder<'a>
    where
        Cow<'a, str>: From<V>,
    {
        self.params.insert(key, value.into());
        self
    }
    /// Returns a `PixivRequest` which can be inspected and/or executed with `Pixiv::execute()`.
    #[inline]
    pub fn build(mut self) -> PixivRequest {
        self.request.extend_query_pairs(&self.params);
        self.request
    }
    /// Sends the request. This function consumes `self`.
    pub fn send(self) -> Result<Response, reqwest::Error> {
        let pixiv = self.pixiv;
        let req = self.build();
        trace!("Request URL: {}", req.url);
        pixiv.execute(req)
    }
}

fn comma_delimited<T, B, I>(iter: I) -> String
where
    T: fmt::Display + ?Sized,
    B: Borrow<T>,
    I: IntoIterator<Item = B>,
{
    let mut iter = iter.into_iter();
    let mut ret = String::new();
    if let Some(b) = iter.next() {
        write!(ret, "{}", b.borrow()).unwrap();
        for b in iter {
            write!(ret, ",{}", b.borrow()).unwrap();
        }
    }
    ret
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_login() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");
    }

    #[test]
    fn test_refresh_auth() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");

        pixiv
            .refresh_auth()
            .expect("Failed to refresh access token");
    }

    #[test]
    fn test_bad_words() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");

        let bad_words: Value = pixiv
            .bad_words()
            .send()
            .expect("Request failed.")
            .json()
            .expect("Failed to parse as json.");

        println!("{}", bad_words);
    }

    #[test]
    fn test_work() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");

        let work: Value = pixiv
            .work(66024340)
            .send()
            .expect("Request failed.")
            .json()
            .expect("Failed to parse as json.");

        println!("{}", work);
    }

    #[test]
    fn test_user() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");

        let following_works: Value = pixiv
            .user(6996493)
            .send()
            .expect("Request failed.")
            .json()
            .expect("Failed to parse as json.");

        println!("{}", following_works);
    }

    #[test]
    fn test_following_works() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        kankyo::load().unwrap();

        pixiv
            .login(
                &kankyo::key("PIXIV_ID").expect("PIXIV_ID isn't set!"),
                &kankyo::key("PIXIV_PW").expect("PIXIV_PW isn't set!"),
            )
            .expect("Failed to log in.");

        let following_works: Value = pixiv
            .following_works()
            .image_sizes(&["large"])
            .include_sanity_level(false)
            .send()
            .expect("Request failed.")
            .json()
            .expect("Failed to parse as json.");

        println!("{}", following_works);
    }

    // Test that generic method calls compile properly.
    #[test]
    fn test_into_iterator() {
        let client = Client::new();
        let pixiv = Pixiv::new(&client);

        let slice: &[usize] = &[0, 1, 2];
        let vec = slice.to_owned();
        let iter = vec.clone().into_iter().chain(Some(3));

        pixiv.favorite_works_remove(slice);
        pixiv.favorite_works_remove(vec.clone());
        pixiv.favorite_works_remove(iter.clone());

        pixiv.following_remove(slice);
        pixiv.following_remove(vec);
        pixiv.following_remove(iter);
    }

    #[test]
    #[should_panic]
    fn test_login_fail() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        pixiv.login("", "").expect("Failed to log in.");
    }
}

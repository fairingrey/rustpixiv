//! # pixiv
//!
//! The `pixiv` crate provides an unofficial library for the Pixiv API.
//!
//! This crate uses the crates `reqwest` and `serde_json`.
//!
//! ## Logging In
//!
//! To log in, you need to create a new `Pixiv` struct and pass in a `reqwest::Client` as such:
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # extern crate serde;
//! # extern crate serde_json;
//! # use pixiv::Pixiv;
//! # use reqwest::Client;
//! # use serde_json::Value;
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
//! ## Making a Request
//!
//! This crate relies on the builder pattern for using and modifying a request. A typical request may look like this:
//!
//! ```rust,no_run
//! # extern crate pixiv;
//! # extern crate reqwest;
//! # extern crate serde;
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
//! # extern crate serde;
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
//!        .image_sizes(vec!["large"])
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
extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;

use std::collections::HashMap;

use chrono::naive::NaiveDate;

use reqwest::header::{Authorization, Bearer, Headers, Referer};
use reqwest::Client;
use reqwest::Response;
use reqwest::StatusCode;
use reqwest::Url;

use serde_json::Value;

// This is taken from the Android app, don't worry about it. It's not really "compromisable", to some degree.
const CLIENT_ID: &'static str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &'static str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";

/// Used to authenticate to the Pixiv servers and construct Pixiv requests through methods creating `PixivRequestBuilder`.
#[derive(Debug, Clone)]
pub struct Pixiv<'a> {
    client: &'a Client,
    access_token: String,
    refresh_token: String,
}

/// Pixiv request. You can create this using `PixivRequestBuilder::Build`. This is for if you wish to inspect the request before sending.
#[derive(Debug, Clone)]
pub struct PixivRequest {
    method: Method,
    url: Url,
    headers: Headers,
}

/// Pixiv request builder. You can create this using any of the provided methods in `Pixiv`, or through `PixivRequestBuilder::new`.
#[derive(Debug, Clone)]
pub struct PixivRequestBuilder<'a> {
    client: &'a Client,
    method: Method,
    url: Url,
    headers: Headers,
    params: HashMap<&'a str, String>,
}

/// Enum for which HTTP method to use.
#[derive(Debug, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    DELETE,
}

/// Enum to set publicity param.
#[derive(Debug, Clone, Copy)]
pub enum Publicity {
    Public,
    Private,
}

impl Publicity {
    fn as_str(&self) -> &str {
        match self {
            &Publicity::Public => "public",
            &Publicity::Private => "private",
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
    fn as_str(&self) -> &str {
        match self {
            &RankingType::All => "all",
            &RankingType::Illust => "illust",
            &RankingType::Manga => "manga",
            &RankingType::Ugoira => "ugoira",
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
    fn as_str(&self) -> &str {
        match self {
            &RankingMode::Daily => "daily",
            &RankingMode::Weekly => "weekly",
            &RankingMode::Monthly => "monthly",
            &RankingMode::Rookie => "rookie",
            &RankingMode::Original => "original",
            &RankingMode::Male => "male",
            &RankingMode::Female => "female",
            &RankingMode::DailyR18 => "daily_r18",
            &RankingMode::WeeklyR18 => "weekly_r18",
            &RankingMode::MaleR18 => "male_r18",
            &RankingMode::FemaleR18 => "female_r18",
            &RankingMode::R18G => "r18g",
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
    fn as_str(&self) -> &str {
        match self {
            &SearchPeriod::All => "all",
            &SearchPeriod::Day => "day",
            &SearchPeriod::Week => "week",
            &SearchPeriod::Month => "month",
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
    fn as_str(&self) -> &str {
        match self {
            &SearchMode::Text => "text",
            &SearchMode::Tag => "tag",
            &SearchMode::ExactTag => "exact_tag",
            &SearchMode::Caption => "caption",
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
    fn as_str(&self) -> &str {
        match self {
            &SearchOrder::Descending => "desc",
            &SearchOrder::Ascending => "asc",
        }
    }
}

impl<'a> Pixiv<'a> {
    /// Creates a new Pixiv struct.
    pub fn new(client: &Client) -> Pixiv {
        Pixiv {
            client: client,
            access_token: String::default(),
            refresh_token: String::default(),
        }
    }
    /// This is required to use all the other functions this library provides. Requires a valid username and password.
    pub fn login(&mut self, username: &str, password: &str) {
        let mut data = HashMap::new();

        data.insert("client_id", CLIENT_ID);
        data.insert("client_secret", CLIENT_SECRET);
        data.insert("get_secure_url", "1");

        data.insert("grant_type", "password");
        data.insert("username", username);
        data.insert("password", password);

        let mut res = self.send_auth_request(data)
            .expect("Error occured while requesting token.");

        match res.status() {
            StatusCode::Ok | StatusCode::MovedPermanently | StatusCode::Found => {
                // success
            }
            s => eprintln!(
                "Login failed. Check your username and password. Response: {:?}",
                s
            ),
        }

        let json_response: Value = serde_json::from_str(&res.text().unwrap()).unwrap();

        self.access_token = String::from(
            json_response["response"]["access_token"]
                .as_str()
                .expect("Failed to get access token."),
        );
        self.refresh_token = String::from(
            json_response["response"]["refresh_token"]
                .as_str()
                .expect("Failed to get refresh token."),
        );
    }
    /// Refreshes the authentication. You should use this when your access token is close to expiring.
    pub fn refresh_auth(&mut self) {
        let refresh_clone = self.refresh_token.clone();
        let mut data = HashMap::new();

        data.insert("client_id", CLIENT_ID);
        data.insert("client_secret", CLIENT_SECRET);
        data.insert("get_secure_url", "1");

        data.insert("grant_type", "refresh_token");
        data.insert("refresh_token", refresh_clone.as_str());

        let mut res = self.send_auth_request(data)
            .expect("Error occured while requesting token.");

        match res.status() {
            StatusCode::Ok | StatusCode::MovedPermanently | StatusCode::Found => {
                // success
            }
            s => eprintln!("Login failed. Check your refresh_token. Response: {:?}", s),
        }

        let json_response: Value = serde_json::from_str(&res.text().unwrap()).unwrap();

        println!("{}", json_response);

        self.access_token = String::from(
            json_response["response"]["access_token"]
                .as_str()
                .expect("Failed to get access token."),
        );
        self.refresh_token = String::from(
            json_response["response"]["refresh_token"]
                .as_str()
                .expect("Failed to get refresh token."),
        );
    }
    // private helper method
    fn send_auth_request(&self, data: HashMap<&str, &str>) -> Result<Response, reqwest::Error> {
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
            self.client,
            self.access_token.to_owned(),
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
    }
    /// Used to build a request to favorite a work on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_work_add(&self, work_id: usize) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let mut params = HashMap::new();
        params.insert("work_id", work_id.to_string());
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::POST,
            url,
            params,
        )
    }
    /// Used to build a request to remove favorited works on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_works_remove(&self, work_ids: Vec<usize>) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let mut params = HashMap::new();
        params.insert(
            "ids",
            work_ids
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
        );
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
    }
    /// Used to build a request to follow a user on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_add(&self, user_id: usize) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let mut params = HashMap::new();
        params.insert("target_user_id", user_id.to_string());
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::POST,
            url,
            params,
        )
    }
    /// Used to build a request to unfollow a user on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_remove(&self, user_ids: Vec<usize>) -> PixivRequestBuilder {
        let url = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let extra_params = [("publicity", "public")];
        let url = Url::parse(&url).unwrap();
        let mut params = HashMap::new();
        params.insert(
            "delete_ids",
            user_ids
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
        );
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let extra_params = vec![
            ("mode", "daily"),
            ("page", "1"),
            ("per_page", "50"),
            ("include_stats", "True"),
            ("include_sanity_level", "True"),
            ("image_sizes", "px_128x128,small,medium,large,px_480mw"),
            ("profile_image_sizes", "px_170x170,px_50x50"),
        ];
        let url = Url::parse(&url).unwrap();
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
    pub fn search_works(&self, query: &str) -> PixivRequestBuilder {
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
        let mut params = HashMap::new();
        params.insert("q", String::from(query));
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
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
        let mut params = HashMap::new();
        for p in extra_params.into_iter() {
            params.insert(p.0, String::from(p.1));
        }
        PixivRequestBuilder::new(
            self.client,
            self.access_token.to_owned(),
            Method::GET,
            url,
            params,
        )
    }
    /// Executes a given `PixivRequest`.
    pub fn execute(&self, request: PixivRequest) -> Result<Response, reqwest::Error> {
        match request.method {
            Method::GET => self.client.get(request.url).headers(request.headers).send(),
            Method::POST => self.client
                .post(request.url)
                .headers(request.headers)
                .send(),
            Method::DELETE => self.client
                .delete(request.url)
                .headers(request.headers)
                .send(),
        }
    }
}

impl PixivRequest {
    /// Create a new `PixivRequest`.
    /// A `PixivRequest` is returned when calling `build()` on `PixivRequestBuilder`, so it is recommended you use that instead.
    pub fn new(method: Method, url: Url, headers: Headers) -> PixivRequest {
        PixivRequest {
            method: method,
            url: url,
            headers: headers,
        }
    }
    /// Get the method.
    pub fn method(&self) -> &Method {
        &self.method
    }
    /// Get a mutable reference to the method.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }
    /// Get the url.
    pub fn url(&self) -> &Url {
        &self.url
    }
    /// Get a mutable reference to the url.
    pub fn url_mut(&mut self) -> &Url {
        &mut self.url
    }
    /// Get the headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    /// Get a mutable reference to the headers.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }
}

impl<'a> PixivRequestBuilder<'a> {
    /// Create a new `PixivRequestBuilder`.
    /// Functions in `Pixiv` expedite a lot of this for you, so using this directly isn't recommended unless you know what you want.
    pub fn new(
        client: &'a Client,
        access_token: String,
        method: Method,
        url: Url,
        params: HashMap<&'a str, String>,
    ) -> PixivRequestBuilder<'a> {
        // set headers
        let mut headers = Headers::new();
        headers.set(Referer::new("http://spapi.pixiv.net/"));
        headers.set(Authorization(Bearer {
            token: access_token,
        }));
        PixivRequestBuilder {
            client: client,
            method: method,
            url: url,
            headers: headers,
            params: params,
        }
    }
    /// Sets the `page` param.
    pub fn page(mut self, value: usize) -> PixivRequestBuilder<'a> {
        self.params.insert("page", value.to_string());
        self
    }
    /// Sets the `per_page` param.
    pub fn per_page(mut self, value: usize) -> PixivRequestBuilder<'a> {
        self.params.insert("value", value.to_string());
        self
    }
    /// Sets the `max_id` param.
    pub fn max_id(mut self, value: usize) -> PixivRequestBuilder<'a> {
        self.params.insert("max_id", value.to_string());
        self
    }
    /// Sets the `image_sizes` param. Available types: `px_128x128`, `small`, `medium`, `large`, `px_480mw`
    pub fn image_sizes(mut self, values: Vec<&str>) -> PixivRequestBuilder<'a> {
        self.params.insert("image_sizes", values.join(","));
        self
    }
    /// Sets the `profile_image_sizes` param. Available types: `px_170x170,px_50x50`
    pub fn profile_image_sizes(mut self, values: Vec<&str>) -> PixivRequestBuilder<'a> {
        self.params.insert("profile_image_sizes", values.join(","));
        self
    }
    /// Sets the `publicity` param. Must be a value of enum `Publicity`.
    pub fn publicity(mut self, value: Publicity) -> PixivRequestBuilder<'a> {
        self.params
            .insert("publicity", String::from(value.as_str()));
        self
    }
    /// Sets the `show_r18` param. `true` means R-18 works will be included.
    pub fn show_r18(mut self, value: bool) -> PixivRequestBuilder<'a> {
        match value {
            true => self.params.insert("show_r18", String::from("1")),
            false => self.params.insert("show_r18", String::from("0")),
        };
        self
    }
    /// Sets the `include_stats` param.
    pub fn include_stats(mut self, value: bool) -> PixivRequestBuilder<'a> {
        match value {
            true => self.params.insert("include_stats", String::from("true")),
            false => self.params.insert("include_stats", String::from("false")),
        };
        self
    }
    /// Sets the `include_sanity_level` param.
    pub fn include_sanity_level(mut self, value: bool) -> PixivRequestBuilder<'a> {
        match value {
            true => self.params
                .insert("include_sanity_level", String::from("true")),
            false => self.params
                .insert("include_sanity_level", String::from("false")),
        };
        self
    }
    /// Sets the ranking mode in the case of a `ranking()` call. Must be a value of enum `RankingMode`.
    pub fn ranking_mode(mut self, value: RankingMode) -> PixivRequestBuilder<'a> {
        self.params.insert("mode", String::from(value.as_str()));
        self
    }
    /// Sets the `date` param. Must be a valid date in the form of `%Y-%m-%d`, e.g. `2018-2-22`.
    pub fn date(mut self, value: &'a str) -> PixivRequestBuilder<'a> {
        // just to validate the date format
        NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("Invalid date or format given.");
        self.params.insert("date", String::from(value));
        self
    }
    /// Sets the `period` param in the case of a `search_works()` call. Must be a value of enum `SearchPeriod`.
    pub fn search_period(mut self, value: SearchPeriod) -> PixivRequestBuilder<'a> {
        self.params.insert("period", String::from(value.as_str()));
        self
    }
    /// Sets the `mode` param in the case of a `search_works()` call. Must be a value of enum `SearchMode`.
    pub fn search_mode(mut self, value: SearchMode) -> PixivRequestBuilder<'a> {
        self.params.insert("mode", String::from(value.as_str()));
        self
    }
    /// Sets the `order` param in the case of a `search_works()` call. Must be a value of enum `SearchOrder`.
    pub fn search_order(mut self, value: SearchOrder) -> PixivRequestBuilder<'a> {
        self.params.insert("order", String::from(value.as_str()));
        self
    }
    /// Sets the `sort` param in the case of a `search_works()` call. Not sure if there's any variations here, but this function is included for convenience.
    pub fn search_sort(mut self, value: &str) -> PixivRequestBuilder<'a> {
        self.params.insert("sort", String::from(value));
        self
    }
    /// Sets the `types` param in the case of a `search_works()` call. Available values: `illustration`, `manga`, `ugoira`.
    pub fn search_types(mut self, values: Vec<&str>) -> PixivRequestBuilder<'a> {
        self.params.insert("types", values.join(","));
        self
    }
    fn url_with_params(&self) -> Url {
        Url::parse_with_params(self.url.as_str(), &self.params).unwrap()
    }
    /// Returns a `PixivRequest` which can be inspected and/or executed with `Pixiv::execute()`.
    pub fn build(self) -> PixivRequest {
        PixivRequest {
            method: self.method,
            url: self.url_with_params(),
            headers: self.headers,
        }
    }
    /// Sends a request. This function consumes `self`.
    pub fn send(self) -> Result<Response, reqwest::Error> {
        let url = self.url_with_params();
        trace!("Request URL: {}", url);
        match self.method {
            Method::GET => self.client.get(url).headers(self.headers).send(),
            Method::POST => self.client.post(url).headers(self.headers).send(),
            Method::DELETE => self.client.delete(url).headers(self.headers).send(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::env; // for debugging, PIXIV_ID and PIXIV_PW need to be set in env

    #[test]
    fn test_login() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );
    }

    #[test]
    fn test_refresh_auth() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );

        pixiv.refresh_auth();
    }

    #[test]
    fn test_bad_words() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );

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

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );

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

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );

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

        pixiv.login(
            &env::var("PIXIV_ID").expect("PIXIV_ID isn't set!"),
            &env::var("PIXIV_PW").expect("PIXIV_PW isn't set!"),
        );

        let following_works: Value = pixiv
            .following_works()
            .image_sizes(vec!["large"])
            .include_sanity_level(false)
            .send()
            .expect("Request failed.")
            .json()
            .expect("Failed to parse as json.");

        println!("{}", following_works);
    }

    #[test]
    #[should_panic]
    fn test_login_fail() {
        let client = Client::new();

        let mut pixiv: Pixiv = Pixiv::new(&client);

        pixiv.login("", "");
    }
}

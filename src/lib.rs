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
//! # use pixiv::client::Pixiv;
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
//! # use pixiv::client::Pixiv;
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
//! # use pixiv::client::Pixiv;
//! # use pixiv::PixivRequestBuilder;
//! # use reqwest::Client;
//! # use serde_json::Value;
//! # fn main() {
//! #   let client = Client::new();
//! #   let mut pixiv: Pixiv = Pixiv::new(&client);
//! #   pixiv.login("username", "password");
//!     let request = PixivRequestBuilder::work(66024340).build();
//!     let work: Value = pixiv
//!         .execute(request)
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
//! # use pixiv::client::Pixiv;
//! # use pixiv::PixivRequestBuilder;
//! # use reqwest::Client;
//! # use serde_json::Value;
//! # fn main() {
//! #   let client = Client::new();
//! #   let mut pixiv: Pixiv = Pixiv::new(&client);
//! #   pixiv.login("username", "password");
//!     let request = PixivRequestBuilder::following_works()
//!        .image_sizes(&["large"])
//!        .include_sanity_level(false)
//!        .build();
//!     let following_works: Value = pixiv
//!        .execute(request)
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
#[cfg(feature = "reqwest-client")]
pub extern crate reqwest;
pub extern crate http;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate bytes;

#[cfg(test)]
extern crate kankyo;

use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Write;

use chrono::naive::NaiveDate;

pub use http::{HttpTryFrom, header, HeaderMap, Method, uri::Uri};

mod utils;
#[cfg(feature = "reqwest-client")]
pub mod client;

use utils::comma_delimited;

/// Pixiv request. You can create this using `PixivRequestBuilder::build`. This is for if you wish to inspect the request before sending.
#[derive(Debug, Clone)]
pub struct PixivRequest {
    pub method: Method,
    pub url: Uri,
    pub headers: HeaderMap,
}

/// Pixiv request builder. You can create this using any of the provided methods in `Pixiv`, or through `PixivRequestBuilder::new`.
#[derive(Debug, Clone)]
pub struct PixivRequestBuilder<'a> {
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

impl PixivRequest {
    /// Create a new `PixivRequest`.
    /// A `PixivRequest` is returned when calling `build()` on `PixivRequestBuilder`, so it is recommended you use that instead.
    #[inline]
    pub fn new(method: Method, url: Uri, headers: HeaderMap) -> PixivRequest {
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
    pub fn url(&self) -> &Uri {
        &self.url
    }
    /// Get a mutable reference to the url.
    #[inline]
    pub fn url_mut(&mut self) -> &mut Uri {
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

    ///Sets query using `serde_urlencoded`
    fn set_query_params<Q: serde::Serialize>(mut self, params: &Q) -> Self {
        let mut uri_parts = self.url.into_parts();
        let path = uri_parts.path_and_query;

        let mut buffer = utils::BytesWriter::with_smol_capacity();
        let query = serde_urlencoded::to_string(params).expect("To url-encode");

        let _ = match path {
            Some(path) => write!(buffer, "{}?{}", path.path(), query),
            None => write!(buffer, "?{}", query),
        };

        uri_parts.path_and_query = Some(http::uri::PathAndQuery::from_shared(buffer.into_inner().freeze()).expect("To create path and query"));

        self.url = match http::Uri::from_parts(uri_parts) {
            Ok(uri) => uri,
            Err(error) => panic!("Unable to set query for URI: {}", error)
        };

        self
    }
}

impl<'a> PixivRequestBuilder<'a> {
    /// Create a new `PixivRequestBuilder`.
    /// Functions in `Pixiv` expedite a lot of this for you, so using this directly isn't recommended unless you know what you want.
    pub fn new(method: Method, url: Uri, params: HashMap<&'a str, Cow<'a, str>>) -> Self {
        // set headers
        let mut headers = HeaderMap::new();
        headers.insert(header::REFERER, header::HeaderValue::from_static("http://spapi.pixiv.net/"));

        PixivRequestBuilder {
            request: PixivRequest::new(method, url, headers),
            params,
        }
    }
    /// Sets the `page` param.
    #[inline]
    pub fn page(self, value: usize) -> Self {
        self.raw_param("page", value.to_string())
    }
    /// Sets the `per_page` param.
    #[inline]
    pub fn per_page(self, value: usize) -> Self {
        self.raw_param("value", value.to_string())
    }
    /// Sets the `max_id` param.
    #[inline]
    pub fn max_id(self, value: usize) -> Self {
        self.raw_param("max_id", value.to_string())
    }
    /// Sets the `image_sizes` param. Available types: `px_128x128`, `small`, `medium`, `large`, `px_480mw`
    #[inline]
    pub fn image_sizes(self, values: &[&str]) -> Self {
        self.raw_param("image_sizes", comma_delimited::<&str, _, _>(values))
    }
    /// Sets the `profile_image_sizes` param. Available types: `px_170x170,px_50x50`
    #[inline]
    pub fn profile_image_sizes(self, values: &[&str]) -> Self {
        self.raw_param("profile_image_sizes", comma_delimited::<&str, _, _>(values))
    }
    /// Sets the `publicity` param. Must be a value of enum `Publicity`.
    #[inline]
    pub fn publicity(self, value: Publicity) -> Self {
        self.raw_param("publicity", value.as_str())
    }
    /// Sets the `show_r18` param. `true` means R-18 works will be included.
    #[inline]
    pub fn show_r18(self, value: bool) -> Self {
        if value {
            self.raw_param("show_r18", "1")
        } else {
            self.raw_param("show_r18", "0")
        }
    }
    /// Sets the `include_stats` param.
    #[inline]
    pub fn include_stats(self, value: bool) -> Self {
        if value {
            self.raw_param("include_stats", "true")
        } else {
            self.raw_param("include_stats", "false")
        }
    }
    /// Sets the `include_sanity_level` param.
    #[inline]
    pub fn include_sanity_level(self, value: bool) -> Self {
        if value {
            self.raw_param("include_sanity_level", "true")
        } else {
            self.raw_param("include_sanity_level", "false")
        }
    }
    /// Sets the ranking mode in the case of a `ranking()` call. Must be a value of enum `RankingMode`.
    #[inline]
    pub fn ranking_mode(self, value: RankingMode) -> Self {
        self.raw_param("mode", value.as_str())
    }
    /// Sets the `date` param. Must be a valid date in the form of `%Y-%m-%d`, e.g. `2018-2-22`.
    pub fn date<V>(self, value: V) -> Self
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
    pub fn search_period(self, value: SearchPeriod) -> Self {
        self.raw_param("period", value.as_str())
    }
    /// Sets the `mode` param in the case of a `search_works()` call. Must be a value of enum `SearchMode`.
    #[inline]
    pub fn search_mode(self, value: SearchMode) -> Self {
        self.raw_param("mode", value.as_str())
    }
    /// Sets the `order` param in the case of a `search_works()` call. Must be a value of enum `SearchOrder`.
    #[inline]
    pub fn search_order(self, value: SearchOrder) -> Self {
        self.raw_param("order", value.as_str())
    }
    /// Sets the `sort` param in the case of a `search_works()` call. Not sure if there's any variations here, but this function is included for convenience.
    pub fn search_sort<V>(self, value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        self.raw_param("sort", value)
    }
    /// Sets the `types` param in the case of a `search_works()` call. Available values: `illustration`, `manga`, `ugoira`.
    #[inline]
    pub fn search_types(self, values: &[&str]) -> Self {
        self.raw_param("types", comma_delimited::<&str, _, _>(values))
    }
    fn raw_param<V>(mut self, key: &'a str, value: V) -> Self
    where
        Cow<'a, str>: From<V>,
    {
        self.params.insert(key, value.into());
        self
    }

    /// Used to build a request to retrive `bad_words.json`.
    /// # Request Transforms
    /// None
    pub fn bad_words() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1.1/bad_words.json";
        let url = Uri::from_static(API_URL);
        PixivRequestBuilder::new(
            Method::GET,
            url,
            HashMap::default(),
        )
    }
    /// Used to build a request to retrieve information of a work.
    /// # Request Transforms
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    pub fn work(illust_id: usize) -> Self {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/works/{}.json",
            illust_id
        );
        let extra_params = [
            ("image_sizes", "px_128x128,small,medium,large,px_480mw"),
            ("include_stats", "true"),
        ];
        let url = Uri::try_from(url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrieve information of a user.
    /// # Request Transforms
    /// * `profile_image_sizes` (default: `px_170x170,px_50x50`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    pub fn user(user_id: usize) -> Self {
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
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrieve your account's feed.
    /// # Request Transforms
    /// * `show_r18` (default: `true`)
    /// * `max_id`
    pub fn feed() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/feeds.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [
            ("relation", "all"),
            ("type", "touch_nottext"),
            ("show_r18", "1"),
        ];
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrieve works favorited on your account.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `50`)
    /// * `publicity` (default: `public`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    pub fn favorite_works() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [
            ("page", "1"),
            ("per_page", "50"),
            ("publicity", "public"),
            ("image_sizes", "px_128x128,px_480mw,large"),
        ];
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to favorite a work on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_work_add(work_id: usize) -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [("publicity", "public")];
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("work_id", work_id.to_string().into())))
            .collect();
        PixivRequestBuilder::new(Method::POST, url, params)
    }
    /// Used to build a request to remove favorited works on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn favorite_works_remove<B, I>(work_ids: I) -> Self
    where
        B: Borrow<usize>,
        I: IntoIterator<Item = B>,
    {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/favorite_works.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [("publicity", "public")];
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("ids", comma_delimited(work_ids).into())))
            .collect();
        PixivRequestBuilder::new(Method::DELETE, url, params)
    }
    /// Used to build a request to retrieve newest works from whoever you follow on your account.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn following_works() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/following/works.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
        ];
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrieve users you follow.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn following() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/following.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [("page", "1"), ("per_page", "30"), ("publicity", "public")];
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to follow a user on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_add(user_id: usize) -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [("publicity", "public")];
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("target_user_id", user_id.to_string().into())))
            .collect();
        PixivRequestBuilder::new(Method::POST, url, params)
    }
    /// Used to build a request to unfollow users on your account.
    /// # Request Transforms
    /// * `publicity` (default: `public`)
    pub fn following_remove<B, I>(user_ids: I) -> Self
    where
        B: Borrow<usize>,
        I: IntoIterator<Item = B>,
    {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/me/favorite-users.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [("publicity", "public")];
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("delete_ids", comma_delimited(user_ids).into())))
            .collect();
        PixivRequestBuilder::new(Method::DELETE, url, params)
    }
    /// Used to build a request to retrive a list of works submitted by a user.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_stats` (default: `true`)
    /// * `include_sanity_level` (default: `true`)
    pub fn user_works(user_id: usize) -> Self {
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
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrive a list of works favorited by a user.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `image_sizes` (default: `px_128x128,small,medium,large,px_480mw`)
    /// * `include_sanity_level` (default: `true`)
    pub fn user_favorite_works(user_id: usize) -> Self {
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
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrive a user's feed.
    /// # Request Transforms
    /// * `show_r18` (default: `true`)
    pub fn user_feed(user_id: usize) -> Self {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/feeds.json",
            user_id
        );
        let extra_params = [
            ("relation", "all"),
            ("type", "touch_nottext"),
            ("show_r18", "1"),
        ];
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Used to build a request to retrieve users a user follows.
    /// # Request Transforms
    /// * `page` (default: `1`)
    /// * `per_page` (default: `30`)
    /// * `max_id`
    pub fn user_following(user_id: usize) -> Self {
        let url = format!(
            "https://public-api.secure.pixiv.net/v1/users/{}/following.json",
            user_id
        );
        let extra_params = [("page", "1"), ("per_page", "30")];
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
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
    pub fn ranking(ranking_type: RankingType) -> Self {
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
        let url = Uri::try_from(&url).unwrap();
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
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
    pub fn search_works<V>(query: V) -> PixivRequestBuilder<'a>
    where
        Cow<'a, str>: From<V>,
    {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/search/works.json";
        let url = Uri::from_static(API_URL);

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
        let params = extra_params
            .iter()
            .map(|&(k, v)| (k, v.into()))
            .chain(Some(("q", query.into())))
            .collect();
        PixivRequestBuilder::new(Method::GET, url, params)
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
    pub fn latest_works() -> Self {
        const API_URL: &'static str = "https://public-api.secure.pixiv.net/v1/works.json";
        let url = Uri::from_static(API_URL);

        let extra_params = [
            ("page", "1"),
            ("per_page", "30"),
            ("include_stats", "true"),
            ("include_sanity_level", "true"),
            ("image_sizes", "px_128x128,px480mw,large"),
            ("profile_image_sizes", "px_170x170,px_50x50"),
        ];
        let params = extra_params.iter().map(|&(k, v)| (k, v.into())).collect();
        PixivRequestBuilder::new(Method::GET, url, params)
    }
    /// Returns a `PixivRequest` which can be inspected and/or executed with `Pixiv::execute()`.
    #[inline]
    pub fn build(self) -> PixivRequest {
        self.request.set_query_params(&self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that generic method calls compile properly.
    #[test]
    fn test_into_iterator() {
        let slice: &[usize] = &[0, 1, 2];
        let vec = slice.to_owned();
        let iter = vec.clone().into_iter().chain(Some(3));

        PixivRequestBuilder::favorite_works_remove(slice);
        PixivRequestBuilder::favorite_works_remove(vec.clone());
        PixivRequestBuilder::favorite_works_remove(iter.clone());

        PixivRequestBuilder::following_remove(slice);
        PixivRequestBuilder::following_remove(vec);
        PixivRequestBuilder::following_remove(iter);
    }
}

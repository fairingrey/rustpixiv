use ::std::borrow::{Borrow, Cow};
use ::std::collections::HashMap;

use ::reqwest;
use ::reqwest::{Url, Response, Client};
use ::http::Method;
use ::http::status::StatusCode;
use ::serde_json::Value;

use utils::comma_delimited;
use super::{RankingType, AuthError, PixivRequest, PixivRequestBuilder};

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



use ::std::collections::HashMap;

use ::reqwest;
use ::reqwest::{Response, Client};
use ::http::status::StatusCode;
use ::serde_json::Value;

use super::{AuthError, PixivRequest};

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

    /// Executes a given `PixivRequest`.
    pub fn execute(&self, request: PixivRequest) -> Result<Response, reqwest::Error> {
        let uri = format!("{}", request.url);
        let url = reqwest::Url::parse(&uri).unwrap();
        self.client.request(request.method,  url)
                   .headers(request.headers)
                   .bearer_auth(self.access_token.clone())
                   .send()
    }
}

#[cfg(test)]
mod tests {
    use ::reqwest::Client;
    use ::serde_json::Value;
    use super::Pixiv;

    use super::super::*;

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

        let request = PixivRequestBuilder::bad_words().build();
        let bad_words: Value = pixiv.execute(request)
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

        let request = PixivRequestBuilder::work(66024340).build();
        let work: Value = pixiv.execute(request)
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

        let request = PixivRequestBuilder::user(6996493).build();
        let following_works: Value = pixiv
            .execute(request)
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

        let request = PixivRequestBuilder::following_works()
            .image_sizes(&["large"])
            .include_sanity_level(false)
            .build();
        let following_works: Value = pixiv
            .execute(request)
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

        pixiv.login("", "").expect("Failed to log in.");
    }
}

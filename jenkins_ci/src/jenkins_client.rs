use std::time::Duration;

use reqwest::blocking::{Client, Response};

/* -- Typedefs -- */

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/* -- JenkinsClient definition -- */

pub struct JenkinsClient {
    client: Client,
    user: String,
    token: String,
    crumb: Option<String>,
}

/* -- JenkinsClient public implementation -- */

impl JenkinsClient {
    pub fn new(user: &str, token: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("ci-tools/0.1")
            .cookie_store(true)
            .tls_built_in_root_certs(true)
            .timeout(Some(Duration::from_secs(60)))
            .build()?;

        Ok(Self {
            client,
            user: user.to_string(),
            token: token.to_string(),
            crumb: None,
        })
    }

    #[inline]
    pub fn get(&self, url: &str) -> Result<Response> {
        Ok(self
            .client
            .get(url)
            .basic_auth(&self.user, Some(&self.token))
            .send()?)
    }

    pub fn post(&self, url: &str, parameters: Option<&str>) -> Result<Response> {
        let mut builder = self
            .client
            .post(url)
            .basic_auth(&self.user, Some(&self.token));

        if let Some(ref crumb) = self.crumb {
            builder = builder.header("Jenkins-Crumb", crumb);
        }

        if let Some(params) = parameters {
            builder = builder.query(&[("json", params)]);
        }

        Ok(builder.send()?)
    }

    pub fn init_crumb(&mut self, url: &str) -> Result<()> {
        let resp = self
            .client
            .get(format!("{}/crumbIssuer/api/json", url))
            .basic_auth(&self.user, Some(&self.token))
            .send()?;

        let tmp: serde_json::Value =
            serde_json::from_str(&resp.text().unwrap_or_default()).unwrap_or_default();

        self.crumb = tmp["crumb"].as_str().map(|val| val.to_string());

        Ok(())
    }
}

/* -- JenkinsClient traint implementation -- */

impl Default for JenkinsClient {
    fn default() -> Self {
        let client = Client::builder()
            .user_agent("ci-tools/0.1")
            .cookie_store(true)
            .tls_built_in_root_certs(true)
            .build()
            .unwrap();

        Self {
            client,
            user: String::default(),
            token: String::default(),
            crumb: None,
        }
    }
}

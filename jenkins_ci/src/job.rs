use std::fs;
use std::path::Path;

use super::build::Build;
use super::jenkins_client::JenkinsClient;
use super::ParameterList;

/* -- Macros definitions -- */

macro_rules! japi_url {
    ($url : expr, $suf : expr) => {
        &format!("{}/{}/api/json", $url, $suf)
    };
}

/* -- Typedefs -- */

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/* -- BuildRequest enum -- */

pub enum BuildRequest {
    Id(u64),
    FirstBuild,
    LastBuild,
    LastSuccessfulBuild,
    LastUnsuccessfulBuild,
    LastCompletedBuild,
    LastFailedBuild,
    LastStableBuild,
    LastUnstableBuild,
}

impl std::fmt::Display for BuildRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BuildRequest::Id(id) => format!("{}", id),
                BuildRequest::FirstBuild => "firstBuild".to_string(),
                BuildRequest::LastBuild => "lastBuild".to_string(),
                BuildRequest::LastSuccessfulBuild => "lastSuccessfulBuild".to_string(),
                BuildRequest::LastUnsuccessfulBuild => "lastUnsuccessfulBuild".to_string(),
                BuildRequest::LastCompletedBuild => "lastCompletedBuild".to_string(),
                BuildRequest::LastFailedBuild => "lastFailedBuild".to_string(),
                BuildRequest::LastStableBuild => "lastStableBuild".to_string(),
                BuildRequest::LastUnstableBuild => "lastUnstableBuild".to_string(),
            }
        )
    }
}

/* -- Job struct definition -- */

pub struct Job {
    client: JenkinsClient,
    url: String,
    name: String,
}

/* -- Job public implementation -- */

impl Job {
    pub fn new(
        url: &str,
        job_name: &str,
        user: &str,
        token: &str,
        with_crumb: bool,
    ) -> Result<Job> {
        let mut client = JenkinsClient::new(user, token)?;

        if with_crumb {
            client.init_crumb(url)?;
        }

        Ok(Self {
            client,
            url: format!("{}/job/{}", url, job_name),
            name: job_name.to_string(),
        })
    }

    pub fn get_build(&self, request: &str) -> Result<Build> {
        self.get_build_from_url(japi_url!(self.url, request))
    }

    pub fn get_builds(&self) -> Vec<Build> {
        let mut ret = Vec::<Build>::new();

        if let Ok(resp) = self.client.get(japi_url!(self.url, "")) {
            let json: serde_json::Value =
                serde_json::from_str(&resp.text().unwrap_or_default()).unwrap_or_default();
            let array = &json["builds"];
            let mut i = 0;

            while array[i] != serde_json::Value::Null {
                if let Ok(build) = self
                    .get_build_from_url(japi_url!(array[i]["url"].as_str().unwrap_or_default(), ""))
                {
                    ret.push(build)
                }
                i += 1;
            }
        }

        ret
    }

    pub fn download_from_workspace(
        &self,
        ws_path: &str,
        dest_dir: &str,
        filename: Option<&str>,
        is_file: bool,
    ) -> Result<()> {
        let dest_path = Path::new(dest_dir);

        if !dest_path.exists() {
            fs::create_dir_all(dest_path)?;
        }

        let ws_file = if ws_path.is_empty() {
            format!("{}.zip", self.name)
        } else {
            let filename = Path::new(ws_path)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();

            if filename.is_empty() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
            } else if is_file {
                String::from(filename)
            } else {
                format!("{}.zip", filename)
            }
        };

        let content = self
            .client
            .get(&format!("{}/ws/{}/*zip*/{}", self.url, ws_path, ws_file))?
            .bytes()?;

        let dest_path = dest_path.join(if let Some(filename) = filename {
            filename
        } else {
            &ws_file
        });
        fs::write(dest_path, content)?;

        Ok(())
    }

    pub fn trigger_build(&self) -> Result<()> {
        self.client.post(&format!("{}/build", self.url), None)?;
        Ok(())
    }

    pub fn trigger_build_with_params(&self, parameters: &ParameterList) -> Result<()> {
        self.client.post(
            &format!("{}/build", self.url),
            Some(&serde_json::to_string(&parameters)?),
        )?;

        Ok(())
    }

    pub fn trigger_build_with_str_params(&self, parameters: &str) -> Result<()> {
        self.client
            .post(&format!("{}/build", self.url), Some(parameters))?;

        Ok(())
    }

    pub fn cancel_build(&self, id: &str) -> Result<()> {
        self.client
            .post(&format!("{}/{}/stop", self.url, id), None)?;
        Ok(())
    }
}

/* -- Job private implementation -- */

impl Job {
    #[inline]
    fn get_build_from_url(&self, url: &str) -> Result<Build> {
        let resp_body = self.client.get(url)?.text()?;
        let mut build: Build = serde_json::from_str(&resp_body)?;

        build.set_client(&self.client);

        Ok(build)
    }
}

/* -- Job traits implementations -- */

impl Default for Job {
    fn default() -> Self {
        Self {
            client: JenkinsClient::default(),
            url: String::default(),
            name: String::default(),
        }
    }
}

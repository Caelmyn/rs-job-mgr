use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::error;
use super::jenkins_client::JenkinsClient;

/* -- Macro definitions -- */

macro_rules! getter {
    ($var : tt, $type : ty) => {
        pub fn $var(&self) -> $type {
            self.$var
        }
    };
}

macro_rules! getter_ref {
    ($var : tt, $type : ty) => {
        pub fn $var(&self) -> $type {
            &self.$var
        }
    };
}

macro_rules! getters {
    ($impl : tt, [ $($get : tt ( $var : tt, $type : ty )),* ] ) => {
        impl $impl {
            $($get!{$var, $type})*
        }
    };

    ($impl : tt, <$lt : tt>, [ $($get : tt ( $var : tt, $type : ty )),* ] ) => {
        impl<$lt> $impl<$lt> {
            $($get!{$var, $type})*
        }
    };
}

/* -- Typedefs -- */

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/* -- Artefact definition -- */

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
struct Artifact {
    #[serde(default)]
    relative_path: String,
}

/* Traits implementation -- */

impl std::fmt::Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = Path::new(&self.relative_path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        write!(f, "{}", name)
    }
}

/* -- Build definition -- */

#[derive(Deserialize)]
pub struct Build<'a> {
    number: u64,
    building: bool,
    result: String,
    artifacts: Vec<Artifact>,
    timestamp: u64,
    url: String,

    #[serde(skip)]
    client: Option<&'a JenkinsClient>,
}

getters! {Build, <'a>, [
        getter(number, u64),
        getter(building, bool),
        getter(timestamp, u64),
        getter_ref(result, &str),
        getter_ref(url, &str)
    ]
}

/* -- Build public implementation -- */

impl<'a> Build<'a> {
    pub fn download_artifact(
        &self,
        filter: &str,
        dest_dir: &str,
        alt_name: Option<&str>,
    ) -> Result<()> {
        let jenkins_client = match self.client {
            Some(client) => client,
            None => return Ok(()),
        };

        let path = Path::new(dest_dir);

        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        let file_path = path.join(if let Some(alt) = alt_name {
            alt
        } else {
            filter
        });

        let art = self.artifacts.iter().find(|art| art.to_string() == filter);

        let art_url = match art {
            Some(name) => &name.relative_path,
            None => return Err(error::ArtifactNotFound::new(self.number, filter).into()),
        };

        let content = jenkins_client
            .get(&format!("{}/artifact/{}", self.url, art_url))?
            .bytes()?;
        fs::write(file_path, content)?;

        Ok(())
    }

    pub fn cancel(&self) -> Result<()> {
        if let Some(client) = self.client {
            client.post(&format!("{}/stop", self.url), None)?;
        }

        Ok(())
    }

    pub(crate) fn set_client(&mut self, client: &'a JenkinsClient) {
        self.client = Some(client);
    }
}

/* -- Traits implementation -- */

impl std::fmt::Debug for Build<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "build : {}, building : {}, result : {}, artifacts : {:?}, timestamp : {}, url : {}",
            self.number, self.building, self.result, self.artifacts, self.timestamp, self.url
        )
    }
}

impl std::fmt::Display for Build<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tab = String::new();

        for art in self.artifacts.iter() {
            tab.push_str(&format!("\n    {}", art));
        }

        write!(f, "Build #{}:", self.number)?;
        write!(f, "\n  building : {}", self.building)?;
        write!(f, "\n  result : {}", self.result)?;
        write!(
            f,
            "\n  launched at : {} /* TODO : readable date */",
            self.timestamp
        )?;
        write!(f, "\n  build url : {}", self.url)?;
        write!(f, "\n  artifacts : {}", tab)
    }
}

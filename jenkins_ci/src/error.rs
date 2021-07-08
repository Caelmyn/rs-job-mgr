use std::fmt;

/* -- ArtifactNotFound error -- */

#[derive(Clone, Default)]
pub struct ArtifactNotFound {
    build_id: u64,
    art_name: String,
}

impl ArtifactNotFound {
    pub(crate) fn new(build_id: u64, art_name: &str) -> Self {
        Self {
            build_id,
            art_name: String::from(art_name),
        }
    }
}

impl std::error::Error for ArtifactNotFound {}

impl fmt::Debug for ArtifactNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} : [{}:{}]",
            stringify!(Self),
            self.build_id,
            self.art_name
        )
    }
}

impl fmt::Display for ArtifactNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Artifact {} not found in build #{}",
            self.art_name, self.build_id
        )
    }
}

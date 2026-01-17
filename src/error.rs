use std::path::PathBuf;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Generated config file at: {0}.")]
    NoConfig(PathBuf),
    #[error("Couldn't retrieve IP address.")]
    NoIp,
    #[error("Couldn't retrieve records.")]
    NoRecords,
    #[error("Couldn't delete record.")]
    Delete,
    #[error("Couldn't create record.")]
    Create,
    #[error(transparent)]
    Ior(#[from] std::io::Error),
    #[error(transparent)]
    Varr(#[from] std::env::VarError),
    #[error(transparent)]
    Parse(#[from] toml::de::Error),
    #[error(transparent)]
    Serr(#[from] toml::ser::Error),
    #[error(transparent)]
    Ureqr(#[from] ureq::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

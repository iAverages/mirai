use serde::{Deserialize, Serialize};

pub mod git;
pub mod local;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentManagerTypes {
    Local = 0,
    Git = 1,
}

impl From<ContentManagerTypes> for u8 {
    fn from(value: ContentManagerTypes) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for ContentManagerTypes {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ContentManagerTypes::Local),
            1 => Ok(ContentManagerTypes::Git),
            _ => Err(()),
        }
    }
}

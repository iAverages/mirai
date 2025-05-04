pub mod local;

#[derive(Debug)]
pub enum ContentManagerTypes {
    Local = 0,
}

impl Into<u8> for ContentManagerTypes {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for ContentManagerTypes {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ContentManagerTypes::Local),
            _ => Err(()),
        }
    }
}

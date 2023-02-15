

pub enum Error {
    DetourError(retour::Error),
    ModuleNotLoaded,   
}

impl From<retour::Error> for Error {
    fn from(value: retour::Error) -> Self {
        Error::DetourError(value)
    }
}
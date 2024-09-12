use alloc::string::String;

pub type DevResult<T = ()> = Result<T, DevError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevError {
    ReadError,
    WriteError,
    InvalidInput(Option<String>),
    Unsupported,
    NotADirectory,
    IsADirectory,
    PermissionDenied,
    NotFound,
    DirectoryNotEmpty,
    AlreadyExists,
    IoError,
    StorageFull,
    UnexpectedEof,
    WriteZero,
    InvalidData,
    NotAFile
}


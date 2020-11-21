use err_derive::Error;

#[derive(Debug, Error)]
pub enum DolphinError {
    #[error(display = "{}", _0)]
    Discord(#[error(source)] serenity::Error),
    #[error(display = "{}", _0)]
    Io(#[error(source)] std::io::Error),
    #[error(display = "{}", _0)]
    Other(&'static str),
    #[error(display = "{}", _0)]
    Parse(#[error(source)] std::num::ParseIntError),
    #[error(display = "{}", _0)]
    Rcon(#[error(source)] rcon::Error),
}

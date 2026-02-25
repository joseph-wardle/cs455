pub mod io;
pub mod math;
pub mod render;
pub mod scene;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("the rendering pipeline is not wired yet")]
    NotImplemented,
}

pub type AppResult<T> = Result<T, AppError>;

pub fn run() -> AppResult<()> {
    // Step 2 focuses on crate structure only.
    Ok(())
}

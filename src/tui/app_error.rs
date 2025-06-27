pub enum AppErrorLevel {
    Warning,
    Error
}


pub struct AppError {
    pub level: AppErrorLevel,
    pub message: String
}

impl AppError {
    pub fn warning<T: Into<String>>(msg: T) -> Self {
        Self {
            level: AppErrorLevel::Warning,
            message: msg.into()
        }
    }

    pub fn error<T: Into<String>>(msg: T) -> Self {
        Self {
            level: AppErrorLevel::Error,
            message: msg.into()
        }
    }
}

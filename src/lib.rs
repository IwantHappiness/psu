#[derive(Debug, PartialEq, Clone)]
pub struct Password {
    pub service: String,
    pub login: String,
    pub password: String,
}

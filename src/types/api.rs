use serde::Deserialize;

// -------------- Register -----------------
#[derive(Deserialize)]
pub struct RegisterBody {
    pub name: String,
    pub duration: u8,
}

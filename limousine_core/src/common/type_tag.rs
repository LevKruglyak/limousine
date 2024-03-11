use serde::{Deserialize, Serialize};

// Can't use ```&'static str``` here due to deserialization
#[derive(Hash, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TypeTag(pub String);

impl ToString for TypeTag {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub trait GetTypeTag {
    fn tag() -> TypeTag;
}

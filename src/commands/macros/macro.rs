#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Macro {
    _id: mongodb::bson::oid::ObjectId,
    user_id: String,
    name: String,
    command: String,
    args: Option<String>,
}

impl Macro {
    // todo
}

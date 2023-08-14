#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
// the csv header uses PascalCase, while camelCase is used for serializing to json
#[serde(rename_all(deserialize = "PascalCase", serialize = "camelCase"))]
pub struct Person {
    pub id: u64,
    #[serde(rename(deserialize = "Name", serialize = "name"))]
    pub first_name: String,
    pub age: u8,
}

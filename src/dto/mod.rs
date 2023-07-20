use serde::Deserialize;

// 反序列化github api的tag
#[derive(Deserialize)]
pub struct GithubTag {
    pub name: String
}


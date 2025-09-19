use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    pub description: String,
    pub version: String,
    pub tags: Option<Vec<String>>,
    pub platforms: HashMap<String, PlatformDetails>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlatformDetails {
    pub url: String,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
    pub executables: Option<serde_json::Value>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct InstallInfo {
//     pub name: String,
//     pub version: String,
//     pub installed_files: Vec<String>,
//     pub executables: Option<serde_json::Value>,
// }

impl PlatformDetails {
    pub fn get_executables(&self) -> Vec<ExecutableInfo> {
        match &self.executables {
            Some(serde_json::Value::String(path)) => {
                vec![ExecutableInfo {
                    path: path.clone(),
                    name: None,
                }]
            }
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|item| match item {
                    serde_json::Value::String(path) => Some(ExecutableInfo {
                        path: path.clone(),
                        name: None,
                    }),
                    serde_json::Value::Object(obj) => {
                        let path = obj.get("path")?.as_str()?.to_string();
                        let name = obj.get("name").and_then(|v| v.as_str()).map(String::from);
                        Some(ExecutableInfo { path, name })
                    }
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    }
}

#[derive(Debug)]
pub struct ExecutableInfo {
    pub path: String,
    pub name: Option<String>,
}
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDef {
    pub name: String,
    pub order: usize,
    pub collapsed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupsData {
    pub groups: Vec<GroupDef>,
    pub assignments: HashMap<String, String>,
}

pub struct GroupsManager {
    pub groups: Vec<GroupDef>,
    pub assignments: HashMap<String, String>,
    file_path: PathBuf,
}

impl Default for GroupsManager {
    fn default() -> Self {
        let file_path = crate::config::sshm_config_dir()
            .map(|dir| dir.join("groups.json"))
            .unwrap_or_else(|_| PathBuf::from("groups.json"));
        Self {
            groups: Vec::new(),
            assignments: HashMap::new(),
            file_path,
        }
    }
}

impl GroupsManager {
    pub fn load() -> Result<Self> {
        let dir = crate::config::sshm_config_dir()?;
        let file_path = dir.join("groups.json");

        if !file_path.exists() {
            return Ok(Self {
                groups: Vec::new(),
                assignments: HashMap::new(),
                file_path,
            });
        }

        let content = std::fs::read_to_string(&file_path)?;
        let data: GroupsData = serde_json::from_str(&content)?;

        Ok(Self {
            groups: data.groups,
            assignments: data.assignments,
            file_path,
        })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = GroupsData {
            groups: self.groups.clone(),
            assignments: self.assignments.clone(),
        };
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }

    pub fn create_group(&mut self, name: String) {
        if self.groups.iter().any(|g| g.name == name) {
            return;
        }
        let order = self.groups.len();
        self.groups.push(GroupDef {
            name,
            order,
            collapsed: false,
        });
        let _ = self.save();
    }

    #[allow(dead_code)]
    pub fn delete_group(&mut self, name: &str) {
        self.groups.retain(|g| g.name != name);
        self.assignments.retain(|_, v| v != name);
        let _ = self.save();
    }

    pub fn assign_host(&mut self, host: &str, group: &str) {
        self.assignments.insert(host.to_string(), group.to_string());
        let _ = self.save();
    }

    pub fn unassign_host(&mut self, host: &str) {
        self.assignments.remove(host);
        let _ = self.save();
    }

    pub fn toggle_collapse(&mut self, name: &str) {
        if let Some(group) = self.groups.iter_mut().find(|g| g.name == name) {
            group.collapsed = !group.collapsed;
            let _ = self.save();
        }
    }

    pub fn get_group_for_host(&self, host: &str) -> Option<&str> {
        self.assignments.get(host).map(|s| s.as_str())
    }

    pub fn ordered_groups(&self) -> Vec<&GroupDef> {
        let mut sorted: Vec<&GroupDef> = self.groups.iter().collect();
        sorted.sort_by_key(|g| g.order);
        sorted
    }
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamError {
    #[error("Team storage error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Team serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Team error: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: TeamRole,
    pub avatar_color: String,
    pub last_active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    Owner,
    Admin,
    Editor,
    Viewer,
}

impl TeamRole {
    pub fn can_edit(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin | TeamRole::Editor)
    }

    pub fn can_manage(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub members: Vec<TeamMember>,
    pub collection_ids: Vec<String>,
    pub environment_ids: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub user_name: String,
    pub action: String,
    pub resource_type: String,
    pub resource_name: String,
    pub timestamp: String,
}

pub struct TeamManager {
    workspaces: Mutex<Vec<Workspace>>,
    activities: Mutex<Vec<ActivityEvent>>,
    storage_path: Mutex<Option<PathBuf>>,
}

impl TeamManager {
    pub fn new() -> Self {
        Self {
            workspaces: Mutex::new(Vec::new()),
            activities: Mutex::new(Vec::new()),
            storage_path: Mutex::new(None),
        }
    }

    pub fn set_storage_path(&self, path: PathBuf) -> Result<(), TeamError> {
        let persisted: Vec<Workspace> = crate::persistence::load_json_or_default(&path)?;
        *self.workspaces.lock().unwrap_or_else(|e| e.into_inner()) = persisted;
        *self.storage_path.lock().unwrap_or_else(|e| e.into_inner()) = Some(path);
        Ok(())
    }

    fn persist_workspaces(&self, workspaces: &[Workspace]) -> Result<(), TeamError> {
        let path = self
            .storage_path
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(path) = path {
            let json = serde_json::to_string_pretty(workspaces)?;
            crate::persistence::atomic_write(&path, json.as_bytes())?;
        }
        Ok(())
    }

    pub fn list_workspaces(&self) -> Vec<Workspace> {
        self.workspaces
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn get_workspace(&self, id: &str) -> Option<Workspace> {
        self.workspaces
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|w| w.id == id)
            .cloned()
    }

    pub fn create_workspace(
        &self,
        name: &str,
        description: Option<&str>,
        owner: TeamMember,
    ) -> Result<Workspace, TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());

        let workspace = Workspace {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            owner_id: owner.id.clone(),
            members: vec![owner],
            collection_ids: vec![],
            environment_ids: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        workspaces.push(workspace.clone());
        self.persist_workspaces(&workspaces)?;
        Ok(workspace)
    }

    pub fn delete_workspace(&self, id: &str) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let len_before = workspaces.len();
        workspaces.retain(|w| w.id != id);
        if workspaces.len() == len_before {
            return Err(TeamError::NotFound(id.to_string()));
        }
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }

    pub fn add_member(&self, workspace_id: &str, member: TeamMember) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| TeamError::NotFound(workspace_id.to_string()))?;

        if workspace.members.iter().any(|m| m.email == member.email) {
            return Err(TeamError::AlreadyExists(member.email));
        }

        workspace.members.push(member);
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }

    pub fn remove_member(&self, workspace_id: &str, member_id: &str) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| TeamError::NotFound(workspace_id.to_string()))?;

        if workspace.owner_id == member_id {
            return Err(TeamError::PermissionDenied(
                "Cannot remove the owner".to_string(),
            ));
        }

        workspace.members.retain(|m| m.id != member_id);
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }

    pub fn update_member_role(
        &self,
        workspace_id: &str,
        member_id: &str,
        role: TeamRole,
    ) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| TeamError::NotFound(workspace_id.to_string()))?;

        let member = workspace
            .members
            .iter_mut()
            .find(|m| m.id == member_id)
            .ok_or_else(|| TeamError::NotFound(member_id.to_string()))?;

        if workspace.owner_id == member_id && role != TeamRole::Owner {
            return Err(TeamError::PermissionDenied(
                "Cannot change owner's role".to_string(),
            ));
        }

        member.role = role;
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }

    pub fn log_activity(&self, event: ActivityEvent) {
        let mut activities = self.activities.lock().unwrap_or_else(|e| e.into_inner());
        activities.push(event);
        if activities.len() > 200 {
            let drain_count = activities.len() - 200;
            activities.drain(0..drain_count);
        }
    }

    pub fn get_activity(&self, workspace_id: &str, limit: usize) -> Vec<ActivityEvent> {
        self.activities
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|a| a.workspace_id == workspace_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn share_collection(
        &self,
        workspace_id: &str,
        collection_id: &str,
    ) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| TeamError::NotFound(workspace_id.to_string()))?;

        if !workspace
            .collection_ids
            .contains(&collection_id.to_string())
        {
            workspace.collection_ids.push(collection_id.to_string());
        }
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }

    pub fn unshare_collection(
        &self,
        workspace_id: &str,
        collection_id: &str,
    ) -> Result<(), TeamError> {
        let mut workspaces = self.workspaces.lock().unwrap_or_else(|e| e.into_inner());
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .ok_or_else(|| TeamError::NotFound(workspace_id.to_string()))?;

        workspace.collection_ids.retain(|c| c != collection_id);
        self.persist_workspaces(&workspaces)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_owner() -> TeamMember {
        TeamMember {
            id: "owner-1".to_string(),
            name: "Alice".to_string(),
            email: "alice@test.com".to_string(),
            role: TeamRole::Owner,
            avatar_color: "#ff0000".to_string(),
            last_active: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_create_workspace() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test Team", None, owner).unwrap();
        assert_eq!(ws.name, "Test Team");
        assert_eq!(ws.members.len(), 1);
        assert_eq!(ws.members[0].role, TeamRole::Owner);
    }

    #[test]
    fn test_add_remove_member() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        let member = TeamMember {
            id: "member-1".to_string(),
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            role: TeamRole::Editor,
            avatar_color: "#00ff00".to_string(),
            last_active: chrono::Utc::now().to_rfc3339(),
        };

        manager.add_member(&ws.id, member).unwrap();
        let ws = manager.get_workspace(&ws.id).unwrap();
        assert_eq!(ws.members.len(), 2);

        manager.remove_member(&ws.id, "member-1").unwrap();
        let ws = manager.get_workspace(&ws.id).unwrap();
        assert_eq!(ws.members.len(), 1);
    }

    #[test]
    fn test_cannot_remove_owner() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        let result = manager.remove_member(&ws.id, "owner-1");
        assert!(matches!(result, Err(TeamError::PermissionDenied(_))));
    }

    #[test]
    fn test_add_duplicate_member() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager
            .create_workspace("Test", None, owner.clone())
            .unwrap();

        let result = manager.add_member(&ws.id, owner);
        assert!(matches!(result, Err(TeamError::AlreadyExists(_))));
    }

    #[test]
    fn test_update_member_role() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        let member = TeamMember {
            id: "member-1".to_string(),
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            role: TeamRole::Viewer,
            avatar_color: "#00ff00".to_string(),
            last_active: chrono::Utc::now().to_rfc3339(),
        };
        manager.add_member(&ws.id, member).unwrap();

        manager
            .update_member_role(&ws.id, "member-1", TeamRole::Admin)
            .unwrap();

        let ws = manager.get_workspace(&ws.id).unwrap();
        let member = ws.members.iter().find(|m| m.id == "member-1").unwrap();
        assert_eq!(member.role, TeamRole::Admin);
    }

    #[test]
    fn test_cannot_change_owner_role() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        let result = manager.update_member_role(&ws.id, "owner-1", TeamRole::Admin);
        assert!(matches!(result, Err(TeamError::PermissionDenied(_))));
    }

    #[test]
    fn test_share_unshare_collection() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        manager.share_collection(&ws.id, "col-1").unwrap();
        let ws = manager.get_workspace(&ws.id).unwrap();
        assert_eq!(ws.collection_ids.len(), 1);

        manager.unshare_collection(&ws.id, "col-1").unwrap();
        let ws = manager.get_workspace(&ws.id).unwrap();
        assert_eq!(ws.collection_ids.len(), 0);
    }

    #[test]
    fn test_activity_log() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager
            .create_workspace("Test", None, owner.clone())
            .unwrap();

        let event = ActivityEvent {
            id: "evt-1".to_string(),
            workspace_id: ws.id.clone(),
            user_id: owner.id.clone(),
            user_name: owner.name.clone(),
            action: "created_collection".to_string(),
            resource_type: "collection".to_string(),
            resource_name: "My API".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        manager.log_activity(event);
        let activities = manager.get_activity(&ws.id, 10);
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].action, "created_collection");
    }

    #[test]
    fn test_role_permissions() {
        assert!(TeamRole::Owner.can_edit());
        assert!(TeamRole::Admin.can_edit());
        assert!(TeamRole::Editor.can_edit());
        assert!(!TeamRole::Viewer.can_edit());

        assert!(TeamRole::Owner.can_manage());
        assert!(TeamRole::Admin.can_manage());
        assert!(!TeamRole::Editor.can_manage());
        assert!(!TeamRole::Viewer.can_manage());
    }

    #[test]
    fn test_delete_workspace() {
        let manager = TeamManager::new();
        let owner = test_owner();
        let ws = manager.create_workspace("Test", None, owner).unwrap();

        manager.delete_workspace(&ws.id).unwrap();
        assert!(manager.get_workspace(&ws.id).is_none());
    }
}

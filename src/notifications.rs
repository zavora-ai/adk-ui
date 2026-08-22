use crate::surface_runtime::SurfaceRef;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiNotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UiNotification {
    pub severity: UiNotificationSeverity,
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_ref: Option<SurfaceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
}

#[derive(Clone)]
pub struct UiNotificationChannel {
    sender: broadcast::Sender<UiNotification>,
}

impl UiNotificationChannel {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    pub fn publish(&self, notification: UiNotification) -> usize {
        self.sender.send(notification).unwrap_or(0)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UiNotification> {
        self.sender.subscribe()
    }
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for UiNotificationChannel {
    fn default() -> Self {
        Self::new(128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publishes_to_agent_owned_subscribers() {
        let channel = UiNotificationChannel::new(4);
        let mut receiver = channel.subscribe();
        let notification = UiNotification {
            severity: UiNotificationSeverity::Warning,
            title: "Inventory".to_string(),
            body: "Stock is below threshold".to_string(),
            surface_ref: Some(SurfaceRef::new("inventory", 2)),
            action_id: Some("open_inventory".to_string()),
        };
        assert_eq!(channel.publish(notification.clone()), 1);
        assert_eq!(receiver.recv().await.unwrap(), notification);
    }
}

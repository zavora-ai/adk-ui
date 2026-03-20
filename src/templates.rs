//! Pre-built UI Templates
//!
//! A library of ready-to-use UI patterns that agents can render with minimal configuration.
//! Templates provide complete, production-ready layouts for common use cases.

use crate::schema::*;
use std::collections::HashMap;

/// Available UI templates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTemplate {
    /// User registration form with name, email, password
    Registration,
    /// User login form with email and password
    Login,
    /// User profile display card
    UserProfile,
    /// Settings page with form fields
    Settings,
    /// Confirmation dialog for destructive actions
    ConfirmDelete,
    /// System status dashboard with metrics
    StatusDashboard,
    /// Data table with pagination
    DataTable,
    /// Success message card
    SuccessMessage,
    /// Error message card
    ErrorMessage,
    /// Loading state with spinner
    Loading,
}

impl UiTemplate {
    /// Get all available template names
    pub fn all_names() -> &'static [&'static str] {
        &[
            "registration",
            "login",
            "user_profile",
            "settings",
            "confirm_delete",
            "status_dashboard",
            "data_table",
            "success_message",
            "error_message",
            "loading",
        ]
    }

    /// Parse a template name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "registration" | "register" | "signup" => Some(Self::Registration),
            "login" | "signin" => Some(Self::Login),
            "user_profile" | "profile" => Some(Self::UserProfile),
            "settings" | "preferences" => Some(Self::Settings),
            "confirm_delete" | "delete_confirm" => Some(Self::ConfirmDelete),
            "status_dashboard" | "dashboard" | "status" => Some(Self::StatusDashboard),
            "data_table" | "table" => Some(Self::DataTable),
            "success_message" | "success" => Some(Self::SuccessMessage),
            "error_message" | "error" => Some(Self::ErrorMessage),
            "loading" | "spinner" => Some(Self::Loading),
            _ => None,
        }
    }
}

/// Template data that can be customized
#[derive(Debug, Clone, Default)]
pub struct TemplateData {
    /// Custom title
    pub title: Option<String>,
    /// Custom description
    pub description: Option<String>,
    /// User data (name, email, etc.)
    pub user: Option<UserData>,
    /// Key-value data for display
    pub data: HashMap<String, String>,
    /// Status items for dashboard
    pub stats: Vec<StatItem>,
    /// Table columns
    pub columns: Vec<TableColumn>,
    /// Table rows
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    /// Custom message
    pub message: Option<String>,
    /// Theme override
    pub theme: Option<Theme>,
}

/// User data for templates
#[derive(Debug, Clone)]
pub struct UserData {
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
}

/// Status item for dashboard templates
#[derive(Debug, Clone)]
pub struct StatItem {
    pub label: String,
    pub value: String,
    pub status: Option<String>,
}

/// Generate a UI response from a template
pub fn render_template(template: UiTemplate, data: TemplateData) -> UiResponse {
    let components = match template {
        UiTemplate::Registration => registration_template(&data),
        UiTemplate::Login => login_template(&data),
        UiTemplate::UserProfile => user_profile_template(&data),
        UiTemplate::Settings => settings_template(&data),
        UiTemplate::ConfirmDelete => confirm_delete_template(&data),
        UiTemplate::StatusDashboard => status_dashboard_template(&data),
        UiTemplate::DataTable => data_table_template(&data),
        UiTemplate::SuccessMessage => success_message_template(&data),
        UiTemplate::ErrorMessage => error_message_template(&data),
        UiTemplate::Loading => loading_template(&data),
    };

    let mut response = UiResponse::new(components);
    if let Some(theme) = data.theme {
        response = response.with_theme(theme);
    }
    response
}

// --- Template Implementations ---

fn registration_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Card(Card {
        id: Some("registration-card".to_string()),
        title: Some(
            data.title
                .clone()
                .unwrap_or_else(|| "Create Account".to_string()),
        ),
        description: data
            .description
            .clone()
            .or_else(|| Some("Enter your details to register".to_string())),
        content: vec![
            Component::TextInput(TextInput {
                id: Some("name".to_string()),
                name: "name".to_string(),
                label: "Full Name".to_string(),
                placeholder: Some("Enter your name".to_string()),
                input_type: "text".to_string(),
                required: true,
                default_value: None,
                error: None,
                min_length: Some(2),
                max_length: Some(100),
            }),
            Component::TextInput(TextInput {
                id: Some("email".to_string()),
                name: "email".to_string(),
                label: "Email".to_string(),
                placeholder: Some("you@example.com".to_string()),
                input_type: "email".to_string(),
                required: true,
                default_value: None,
                error: None,
                min_length: None,
                max_length: None,
            }),
            Component::TextInput(TextInput {
                id: Some("password".to_string()),
                name: "password".to_string(),
                label: "Password".to_string(),
                placeholder: Some("Choose a strong password".to_string()),
                input_type: "password".to_string(),
                required: true,
                default_value: None,
                error: None,
                min_length: Some(8),
                max_length: None,
            }),
        ],
        footer: Some(vec![Component::Button(Button {
            id: Some("submit".to_string()),
            label: "Create Account".to_string(),
            action_id: "register_submit".to_string(),
            variant: ButtonVariant::Primary,
            disabled: false,
            icon: None,
        })]),
    })]
}

fn login_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Card(Card {
        id: Some("login-card".to_string()),
        title: Some(
            data.title
                .clone()
                .unwrap_or_else(|| "Welcome Back".to_string()),
        ),
        description: data
            .description
            .clone()
            .or_else(|| Some("Sign in to your account".to_string())),
        content: vec![
            Component::TextInput(TextInput {
                id: Some("email".to_string()),
                name: "email".to_string(),
                label: "Email".to_string(),
                placeholder: Some("you@example.com".to_string()),
                input_type: "email".to_string(),
                required: true,
                default_value: None,
                error: None,
                min_length: None,
                max_length: None,
            }),
            Component::TextInput(TextInput {
                id: Some("password".to_string()),
                name: "password".to_string(),
                label: "Password".to_string(),
                placeholder: Some("Enter your password".to_string()),
                input_type: "password".to_string(),
                required: true,
                default_value: None,
                error: None,
                min_length: None,
                max_length: None,
            }),
        ],
        footer: Some(vec![Component::Button(Button {
            id: Some("submit".to_string()),
            label: "Sign In".to_string(),
            action_id: "login_submit".to_string(),
            variant: ButtonVariant::Primary,
            disabled: false,
            icon: None,
        })]),
    })]
}

fn user_profile_template(data: &TemplateData) -> Vec<Component> {
    let user = data.user.as_ref();
    let name = user
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "User".to_string());
    let email = user
        .map(|u| u.email.clone())
        .unwrap_or_else(|| "user@example.com".to_string());
    let role = user
        .and_then(|u| u.role.clone())
        .unwrap_or_else(|| "Member".to_string());

    vec![Component::Card(Card {
        id: Some("profile-card".to_string()),
        title: Some(
            data.title
                .clone()
                .unwrap_or_else(|| "User Profile".to_string()),
        ),
        description: None,
        content: vec![
            Component::Text(Text {
                id: None,
                content: format!("**{}**", name),
                variant: TextVariant::H3,
            }),
            Component::Badge(Badge {
                id: None,
                label: role,
                variant: BadgeVariant::Info,
            }),
            Component::Divider(Divider { id: None }),
            Component::KeyValue(KeyValue {
                id: None,
                pairs: vec![KeyValuePair {
                    key: "Email".to_string(),
                    value: email,
                }],
            }),
        ],
        footer: Some(vec![Component::Button(Button {
            id: Some("edit".to_string()),
            label: "Edit Profile".to_string(),
            action_id: "edit_profile".to_string(),
            variant: ButtonVariant::Secondary,
            disabled: false,
            icon: None,
        })]),
    })]
}

fn settings_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Card(Card {
        id: Some("settings-card".to_string()),
        title: Some(data.title.clone().unwrap_or_else(|| "Settings".to_string())),
        description: data
            .description
            .clone()
            .or_else(|| Some("Manage your preferences".to_string())),
        content: vec![
            Component::Switch(Switch {
                id: Some("notifications".to_string()),
                name: "notifications".to_string(),
                label: "Email Notifications".to_string(),
                default_checked: true,
            }),
            Component::Switch(Switch {
                id: Some("dark_mode".to_string()),
                name: "dark_mode".to_string(),
                label: "Dark Mode".to_string(),
                default_checked: false,
            }),
            Component::Select(Select {
                id: Some("language".to_string()),
                name: "language".to_string(),
                label: "Language".to_string(),
                options: vec![
                    SelectOption {
                        value: "en".to_string(),
                        label: "English".to_string(),
                    },
                    SelectOption {
                        value: "es".to_string(),
                        label: "Spanish".to_string(),
                    },
                    SelectOption {
                        value: "fr".to_string(),
                        label: "French".to_string(),
                    },
                ],
                required: false,
                error: None,
            }),
        ],
        footer: Some(vec![Component::Button(Button {
            id: Some("save".to_string()),
            label: "Save Settings".to_string(),
            action_id: "save_settings".to_string(),
            variant: ButtonVariant::Primary,
            disabled: false,
            icon: None,
        })]),
    })]
}

fn confirm_delete_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Modal(Modal {
        id: Some("confirm-delete-modal".to_string()),
        title: data
            .title
            .clone()
            .unwrap_or_else(|| "Confirm Deletion".to_string()),
        content: vec![Component::Alert(Alert {
            id: None,
            title: "Warning".to_string(),
            description: Some(data.message.clone().unwrap_or_else(|| {
                "This action cannot be undone. All data will be permanently deleted.".to_string()
            })),
            variant: AlertVariant::Warning,
        })],
        footer: Some(vec![
            Component::Button(Button {
                id: Some("cancel".to_string()),
                label: "Cancel".to_string(),
                action_id: "cancel_delete".to_string(),
                variant: ButtonVariant::Secondary,
                disabled: false,
                icon: None,
            }),
            Component::Button(Button {
                id: Some("confirm".to_string()),
                label: "Delete".to_string(),
                action_id: "confirm_delete".to_string(),
                variant: ButtonVariant::Danger,
                disabled: false,
                icon: None,
            }),
        ]),
        size: ModalSize::Small,
        closable: true,
    })]
}

fn status_dashboard_template(data: &TemplateData) -> Vec<Component> {
    let stats = if data.stats.is_empty() {
        vec![
            StatItem {
                label: "CPU".to_string(),
                value: "45%".to_string(),
                status: Some("ok".to_string()),
            },
            StatItem {
                label: "Memory".to_string(),
                value: "78%".to_string(),
                status: Some("warning".to_string()),
            },
            StatItem {
                label: "Disk".to_string(),
                value: "32%".to_string(),
                status: Some("ok".to_string()),
            },
        ]
    } else {
        data.stats.clone()
    };

    vec![
        Component::Text(Text {
            id: None,
            content: data
                .title
                .clone()
                .unwrap_or_else(|| "System Status".to_string()),
            variant: TextVariant::H2,
        }),
        Component::Grid(Grid {
            id: None,
            columns: stats.len().min(4) as u8,
            gap: 4,
            children: stats
                .iter()
                .map(|stat| {
                    let status_variant = match stat.status.as_deref() {
                        Some("ok") | Some("success") => BadgeVariant::Success,
                        Some("warning") => BadgeVariant::Warning,
                        Some("error") | Some("critical") => BadgeVariant::Error,
                        _ => BadgeVariant::Default,
                    };
                    Component::Card(Card {
                        id: None,
                        title: None,
                        description: None,
                        content: vec![
                            Component::Text(Text {
                                id: None,
                                content: stat.label.clone(),
                                variant: TextVariant::Caption,
                            }),
                            Component::Text(Text {
                                id: None,
                                content: stat.value.clone(),
                                variant: TextVariant::H3,
                            }),
                            Component::Badge(Badge {
                                id: None,
                                label: stat.status.clone().unwrap_or_else(|| "ok".to_string()),
                                variant: status_variant,
                            }),
                        ],
                        footer: None,
                    })
                })
                .collect(),
        }),
    ]
}

fn data_table_template(data: &TemplateData) -> Vec<Component> {
    let columns = if data.columns.is_empty() {
        vec![
            TableColumn {
                header: "ID".to_string(),
                accessor_key: "id".to_string(),
                sortable: true,
            },
            TableColumn {
                header: "Name".to_string(),
                accessor_key: "name".to_string(),
                sortable: true,
            },
            TableColumn {
                header: "Status".to_string(),
                accessor_key: "status".to_string(),
                sortable: false,
            },
        ]
    } else {
        data.columns.clone()
    };

    vec![
        Component::Text(Text {
            id: None,
            content: data.title.clone().unwrap_or_else(|| "Data".to_string()),
            variant: TextVariant::H2,
        }),
        Component::Table(Table {
            id: Some("data-table".to_string()),
            columns,
            data: data.rows.clone(),
            sortable: true,
            page_size: Some(10),
            striped: true,
        }),
    ]
}

fn success_message_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Alert(Alert {
        id: Some("success-alert".to_string()),
        title: data.title.clone().unwrap_or_else(|| "Success!".to_string()),
        description: data
            .message
            .clone()
            .or_else(|| Some("Operation completed successfully.".to_string())),
        variant: AlertVariant::Success,
    })]
}

fn error_message_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Alert(Alert {
        id: Some("error-alert".to_string()),
        title: data.title.clone().unwrap_or_else(|| "Error".to_string()),
        description: data
            .message
            .clone()
            .or_else(|| Some("Something went wrong. Please try again.".to_string())),
        variant: AlertVariant::Error,
    })]
}

fn loading_template(data: &TemplateData) -> Vec<Component> {
    vec![Component::Spinner(Spinner {
        id: Some("loading-spinner".to_string()),
        size: SpinnerSize::Large,
        label: data
            .message
            .clone()
            .or_else(|| Some("Loading...".to_string())),
    })]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_template() {
        let response = render_template(UiTemplate::Registration, TemplateData::default());
        assert_eq!(response.components.len(), 1);
    }

    #[test]
    fn test_template_from_name() {
        assert_eq!(
            UiTemplate::from_name("registration"),
            Some(UiTemplate::Registration)
        );
        assert_eq!(
            UiTemplate::from_name("signup"),
            Some(UiTemplate::Registration)
        );
        assert_eq!(UiTemplate::from_name("login"), Some(UiTemplate::Login));
        assert_eq!(UiTemplate::from_name("unknown"), None);
    }
}

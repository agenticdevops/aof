use std::collections::HashMap;

pub mod incident_response;
pub mod monitoring;
pub mod deployment;
pub mod cost_optimization;

/// A template agent within a squad
#[derive(Debug, Clone)]
pub struct TemplateAgent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub avatar: String,
    pub personality_traits: Vec<String>,
    pub skills: Vec<String>,
    pub can: Vec<String>,
    pub cannot: Vec<String>,
}

/// Squad coordination configuration
#[derive(Debug, Clone)]
pub struct SquadConfig {
    pub coordination: String,
    pub communication: String,
}

/// A pre-built squad template
#[derive(Debug, Clone)]
pub struct SquadTemplate {
    pub name: String,
    pub description: String,
    pub agents: Vec<TemplateAgent>,
    pub squad_config: SquadConfig,
    pub customization_hints: Vec<String>,
}

/// Library of all built-in squad templates
pub struct SquadTemplateLibrary {
    templates: HashMap<String, SquadTemplate>,
}

impl SquadTemplateLibrary {
    /// Load all built-in templates
    pub fn load_builtin() -> Self {
        let mut templates = HashMap::new();

        templates.insert(
            "incident-response".to_string(),
            incident_response::template(),
        );
        templates.insert("monitoring".to_string(), monitoring::template());
        templates.insert("deployment".to_string(), deployment::template());
        templates.insert(
            "cost-optimization".to_string(),
            cost_optimization::template(),
        );

        Self { templates }
    }

    /// Get a template by exact name
    pub fn get(&self, name: &str) -> Option<&SquadTemplate> {
        self.templates.get(name)
    }

    /// List all available templates (name, description)
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.templates
            .iter()
            .map(|(name, template)| (name.as_str(), template.description.as_str()))
            .collect()
    }

    /// Find a template by matching keywords in name or description
    pub fn find_by_keywords(&self, keywords: &[&str]) -> Option<&SquadTemplate> {
        for (name, template) in &self.templates {
            for keyword in keywords {
                let keyword_lower = keyword.to_lowercase();
                if name.contains(&keyword_lower)
                    || template
                        .description
                        .to_lowercase()
                        .contains(&keyword_lower)
                {
                    return Some(template);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_4_templates() {
        let library = SquadTemplateLibrary::load_builtin();
        assert_eq!(library.templates.len(), 4);
    }

    #[test]
    fn test_get_by_name() {
        let library = SquadTemplateLibrary::load_builtin();
        let template = library.get("incident-response");
        assert!(template.is_some());
        assert_eq!(template.unwrap().agents.len(), 4);
    }

    #[test]
    fn test_list_returns_4_entries() {
        let library = SquadTemplateLibrary::load_builtin();
        let list = library.list();
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_find_by_keyword_monitoring() {
        let library = SquadTemplateLibrary::load_builtin();
        let template = library.find_by_keywords(&["monitor"]);
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "monitoring");
    }

    #[test]
    fn test_find_by_keyword_incident() {
        let library = SquadTemplateLibrary::load_builtin();
        let template = library.find_by_keywords(&["incident"]);
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "incident-response");
    }

    #[test]
    fn test_all_templates_have_valid_agents() {
        let library = SquadTemplateLibrary::load_builtin();
        for (name, template) in &library.templates {
            assert!(!template.agents.is_empty(), "{} has no agents", name);
            for agent in &template.agents {
                assert!(!agent.id.is_empty(), "{} agent has empty id", name);
                assert!(!agent.name.is_empty(), "{} agent has empty name", name);
                assert!(!agent.role.is_empty(), "{} agent has empty role", name);
                assert!(!agent.avatar.is_empty(), "{} agent has empty avatar", name);
                assert!(
                    !agent.personality_traits.is_empty(),
                    "{} agent has no personality traits",
                    name
                );
                assert!(!agent.skills.is_empty(), "{} agent has no skills", name);
                assert!(!agent.can.is_empty(), "{} agent has no can list", name);
                assert!(!agent.cannot.is_empty(), "{} agent has no cannot list", name);
            }
        }
    }
}

use super::traits::{Specialist, SpecialistOutput};
use crate::types::{IntentClassification, ConversationSession};
use crate::templates::{SquadTemplate, SquadTemplateLibrary};
use aof_llm::Model;
use aof_personas::{Agent, Soul};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct SquadBuilder {
    model: Arc<dyn Model>,
    template_library: SquadTemplateLibrary,
    workspace_path: PathBuf,
}

impl SquadBuilder {
    pub fn new(model: Arc<dyn Model>, workspace_path: PathBuf) -> Self {
        Self {
            model,
            template_library: SquadTemplateLibrary::load_builtin(),
            workspace_path,
        }
    }

    async fn select_template(&self, intent: &IntentClassification) -> Result<&SquadTemplate> {
        // Try to extract squad_type or description from parameters
        let squad_type = intent
            .parameters
            .get("squad_type")
            .or_else(|| intent.parameters.get("type"))
            .and_then(|v| v.as_str());

        let _description = intent
            .parameters
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Try exact match first
        if let Some(st) = squad_type {
            if let Some(template) = self.template_library.get(st) {
                return Ok(template);
            }
        }

        // Try keyword search from parameters
        let keywords: Vec<&str> = intent
            .parameters
            .values()
            .filter_map(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .collect();

        if !keywords.is_empty() {
            if let Some(template) = self.template_library.find_by_keywords(&keywords) {
                return Ok(template);
            }
        }

        // No match found - list available options
        let available = self
            .template_library
            .list()
            .iter()
            .map(|(name, desc)| format!("- **{}**: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n");

        Err(anyhow!(
            "I couldn't determine which squad template to use. Available squads:\n\n{}",
            available
        ))
    }

    fn customize_for_domain(
        &self,
        template: &SquadTemplate,
        domain: &str,
        _description: &str,
    ) -> Result<Vec<(Agent, Soul)>> {
        // MVP: Simple domain customization without Claude
        // Just append domain to role descriptions and personality summaries
        let mut customized_agents = Vec::new();

        for template_agent in &template.agents {
            let customization = serde_json::json!({
                "can": template_agent.can,
                "cannot": template_agent.cannot,
                "personality_summary": format!("{} specialized in {}", template_agent.role, domain),
                "communication_style": "professional",
                "tone": "helpful and focused",
                "values": ["accuracy", "efficiency"],
                "boundaries": template_agent.cannot,
                "default_intro": format!("I'm {}, specialized in {} for {}.", template_agent.name, template_agent.role, domain)
            });

            // Build Agent struct
            let agent = Agent {
                id: template_agent.id.clone(),
                name: template_agent.name.clone(),
                role: template_agent.role.clone(),
                avatar: template_agent.avatar.clone(),
                personality_traits: template_agent.personality_traits.clone(),
                skills: template_agent.skills.clone(),
                can: customization
                    .get("can")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| template_agent.can.clone()),
                cannot: customization
                    .get("cannot")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| template_agent.cannot.clone()),
            };

            // Build Soul struct
            let soul = Soul {
                id: template_agent.id.clone(),
                communication_style: customization
                    .get("communication_style")
                    .and_then(|v| v.as_str())
                    .unwrap_or("professional")
                    .to_string(),
                tone: customization
                    .get("tone")
                    .and_then(|v| v.as_str())
                    .unwrap_or("helpful")
                    .to_string(),
                values: customization
                    .get("values")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["accuracy".to_string(), "efficiency".to_string()]),
                personality_summary: customization
                    .get("personality_summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&template_agent.role)
                    .to_string(),
                boundaries: customization
                    .get("boundaries")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                default_intro: customization
                    .get("default_intro")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Ready to help.")
                    .to_string(),
                communication_guide: format!(
                    "# Communication Style\n\n{}",
                    customization
                        .get("personality_summary")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Professional and focused.")
                ),
            };

            customized_agents.push((agent, soul));
        }

        Ok(customized_agents)
    }

    fn format_agents_yaml(&self, agents: &[(Agent, Soul)]) -> String {
        agents
            .iter()
            .map(|(agent, _)| {
                format!(
                    "- id: {}\n  name: {}\n  role: {}\n  avatar: {}\n  personality_traits:\n{}\n  skills:\n{}\n  can:\n{}\n  cannot:\n{}",
                    agent.id,
                    agent.name,
                    agent.role,
                    agent.avatar,
                    agent.personality_traits.iter().map(|t| format!("    - {}", t)).collect::<Vec<_>>().join("\n"),
                    agent.skills.iter().map(|s| format!("    - {}", s)).collect::<Vec<_>>().join("\n"),
                    agent.can.iter().map(|c| format!("    - {}", c)).collect::<Vec<_>>().join("\n"),
                    agent.cannot.iter().map(|c| format!("    - {}", c)).collect::<Vec<_>>().join("\n"),
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn format_soul_markdown(&self, agents: &[(Agent, Soul)]) -> String {
        let mut output = String::new();
        for (_, soul) in agents {
            output.push_str(&format!("## {}\n\n", soul.id));
            output.push_str("```yaml\n");
            output.push_str(&format!("id: {}\n", soul.id));
            output.push_str(&format!("communication_style: {}\n", soul.communication_style));
            output.push_str(&format!("tone: {}\n", soul.tone));
            output.push_str(&format!("personality_summary: {}\n", soul.personality_summary));
            output.push_str(&format!("values:\n"));
            for value in &soul.values {
                output.push_str(&format!("  - {}\n", value));
            }
            if !soul.boundaries.is_empty() {
                output.push_str(&format!("boundaries:\n"));
                for boundary in &soul.boundaries {
                    output.push_str(&format!("  - {}\n", boundary));
                }
            }
            output.push_str(&format!("default_intro: {}\n", soul.default_intro));
            output.push_str("```\n\n");
            output.push_str(&soul.communication_guide);
            output.push_str("\n\n");
        }
        output
    }

    fn format_squads_yaml(&self, template: &SquadTemplate, agents: &[(Agent, Soul)]) -> String {
        let agent_ids: Vec<String> = agents.iter().map(|(a, _)| a.id.clone()).collect();
        format!(
            "squads:\n  - name: {}\n    agents:\n{}\n    coordination:\n      type: {}\n      communication: {}",
            template.name,
            agent_ids.iter().map(|id| format!("      - {}", id)).collect::<Vec<_>>().join("\n"),
            template.squad_config.coordination,
            template.squad_config.communication
        )
    }
}

#[async_trait]
impl Specialist for SquadBuilder {
    async fn handle(
        &self,
        intent: &IntentClassification,
        _session: &ConversationSession,
    ) -> Result<SpecialistOutput> {
        // Select template
        let template = self.select_template(intent).await?;

        // Check if domain customization is requested
        let domain = intent.parameters.get("domain").and_then(|v| v.as_str());
        let description = intent
            .parameters
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let agents = if let Some(domain_name) = domain {
            // Domain customization
            self.customize_for_domain(template, domain_name, description)?
        } else {
            // Use template defaults
            template
                .agents
                .iter()
                .map(|ta| {
                    let agent = Agent {
                        id: ta.id.clone(),
                        name: ta.name.clone(),
                        role: ta.role.clone(),
                        avatar: ta.avatar.clone(),
                        personality_traits: ta.personality_traits.clone(),
                        skills: ta.skills.clone(),
                        can: ta.can.clone(),
                        cannot: ta.cannot.clone(),
                    };
                    let soul = Soul {
                        id: ta.id.clone(),
                        communication_style: "professional".to_string(),
                        tone: "helpful and focused".to_string(),
                        values: vec!["accuracy".to_string(), "efficiency".to_string()],
                        personality_summary: ta.role.clone(),
                        boundaries: ta.cannot.clone(),
                        default_intro: format!(
                            "I'm {}, your {}. {}",
                            ta.name,
                            ta.role,
                            ta.can.first().unwrap_or(&"Ready to help.".to_string())
                        ),
                        communication_guide: format!(
                            "# Communication Style\n\nI focus on {}.",
                            ta.role
                        ),
                    };
                    (agent, soul)
                })
                .collect()
        };

        // Generate files
        let mut files = HashMap::new();
        files.insert(
            format!("{}/AGENTS.md", self.workspace_path.display()),
            self.format_agents_yaml(&agents),
        );
        files.insert(
            format!("{}/SOUL.md", self.workspace_path.display()),
            self.format_soul_markdown(&agents),
        );
        files.insert(
            format!("{}/squads.yaml", self.workspace_path.display()),
            self.format_squads_yaml(template, &agents),
        );

        let message = if domain.is_some() {
            format!(
                "Created {} squad with {} agents, customized for {} domain.\n\n\
                Agents: {}\n\n\
                Review the generated AGENTS.md, SOUL.md, and squads.yaml files.",
                template.name,
                agents.len(),
                domain.unwrap(),
                agents
                    .iter()
                    .map(|(a, _)| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!(
                "Created {} squad with {} agents.\n\n\
                Agents: {}\n\n\
                Review the generated AGENTS.md, SOUL.md, and squads.yaml files.",
                template.name,
                agents.len(),
                agents
                    .iter()
                    .map(|(a, _)| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        Ok(SpecialistOutput::with_confirmation(files, message))
    }

    fn name(&self) -> &str {
        "squad_builder"
    }
}

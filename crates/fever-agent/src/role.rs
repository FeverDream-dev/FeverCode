use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistRole {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub temperature: Option<f32>,
}

impl SpecialistRole {
    pub fn new(id: String, name: String, description: String, system_prompt: String) -> Self {
        Self {
            id,
            name,
            description,
            system_prompt,
            capabilities: Vec::new(),
            tools: Vec::new(),
            temperature: None,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
}

pub type Role = SpecialistRole;

pub struct RoleRegistry {
    roles: Vec<SpecialistRole>,
}

impl RoleRegistry {
    pub fn new() -> Self {
        let mut registry = Self { roles: Vec::new() };
        registry.load_builtin_roles();
        registry
    }

    fn load_builtin_roles(&mut self) {
        self.register(SpecialistRole::new(
            "coder".to_string(),
            "Coder".to_string(),
            "Code implementation and modification".to_string(),
            "You are a coding specialist. Write clean, efficient, well-documented code. Follow best practices and handle errors appropriately.".to_string(),
        ).with_capabilities(vec!["coding".to_string(), "debugging".to_string()])
         .with_tools(vec!["filesystem".to_string(), "shell".to_string(), "grep".to_string()]));

        self.register(SpecialistRole::new(
            "architect".to_string(),
            "Architect".to_string(),
            "System architecture and design".to_string(),
            "You are an architecture specialist. Design robust, scalable systems. Consider trade-offs and identify patterns.".to_string(),
        ).with_capabilities(vec!["architecture".to_string(), "design".to_string()]));

        self.register(SpecialistRole::new(
            "debugger".to_string(),
            "Debugger".to_string(),
            "Debugging and troubleshooting".to_string(),
            "You are a debugging specialist. Identify root causes, reproduce problems systematically, provide clear explanations of fixes.".to_string(),
        ).with_capabilities(vec!["debugging".to_string(), "analysis".to_string()])
         .with_tools(vec!["shell".to_string(), "grep".to_string()]));

        self.register(SpecialistRole::new(
            "planner".to_string(),
            "Planner".to_string(),
            "Strategic planning and task breakdown".to_string(),
            "You are a planning specialist. Create detailed, actionable plans. Break down work into clear steps, identify dependencies.".to_string(),
        ).with_capabilities(vec!["planning".to_string(), "analysis".to_string()]));

        self.register(SpecialistRole::new(
            "reviewer".to_string(),
            "Reviewer".to_string(),
            "Code review and quality assessment".to_string(),
            "You are a code review specialist. Review code for correctness, style, best practices. Provide constructive feedback.".to_string(),
        ).with_capabilities(vec!["review".to_string(), "quality_assessment".to_string()]));

        self.register(SpecialistRole::new(
            "researcher".to_string(),
            "Researcher".to_string(),
            "Research and information gathering".to_string(),
            "You are a research specialist. Gather comprehensive information, analyze sources, provide well-researched insights.".to_string(),
        ).with_capabilities(vec!["web_search".to_string(), "reading".to_string()])
         .with_tools(vec!["search".to_string()]));

        self.register(SpecialistRole::new(
            "tester".to_string(),
            "Tester".to_string(),
            "Testing and quality assurance".to_string(),
            "You are a testing specialist. Design and implement comprehensive tests. Consider edge cases, ensure reliability.".to_string(),
        ).with_capabilities(vec!["testing".to_string(), "qa".to_string()])
         .with_tools(vec!["shell".to_string()]));

        self.register(SpecialistRole::new(
            "default".to_string(),
            "Default".to_string(),
            "General-purpose coding and assistance".to_string(),
            "You are Fever Code, a senior full-stack developer and coding assistant. You help users build, debug, and improve software projects. You're thorough, practical, and follow best practices.".to_string(),
        ).with_capabilities(vec!["coding".to_string(), "debugging".to_string(), "analysis".to_string()])
         .with_tools(vec!["filesystem".to_string(), "shell".to_string(), "grep".to_string(), "git".to_string()]));

        self.register(SpecialistRole::new(
            "refactorer".to_string(),
            "Refactorer".to_string(),
            "Code refactoring and improvement".to_string(),
            "You are a refactoring specialist. Improve code quality, reduce technical debt, apply modern patterns while preserving functionality.".to_string(),
        ).with_capabilities(vec!["refactoring".to_string(), "code_analysis".to_string()]));

        self.register(SpecialistRole::new(
            "doc_writer".to_string(),
            "Documentation Writer".to_string(),
            "Technical documentation".to_string(),
            "You are a documentation specialist. Write clear, comprehensive technical documentation. Explain complex concepts simply.".to_string(),
        ).with_capabilities(vec!["writing".to_string(), "documentation".to_string()]));
    }

    pub fn register(&mut self, role: SpecialistRole) {
        self.roles.push(role);
    }

    pub fn get(&self, id: &str) -> Option<&SpecialistRole> {
        self.roles.iter().find(|r| r.id == id)
    }

    pub fn list(&self) -> Vec<&SpecialistRole> {
        self.roles.iter().collect()
    }

    pub fn list_ids(&self) -> Vec<String> {
        self.roles.iter().map(|r| r.id.clone()).collect()
    }
}

impl Default for RoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

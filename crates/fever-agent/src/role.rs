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
            "researcher".to_string(),
            "Researcher".to_string(),
            "Deep research and information gathering".to_string(),
            "You are a research specialist. Your task is to gather comprehensive information, analyze sources, and provide well-researched insights. Be thorough and cite sources when possible.".to_string(),
        )
        .with_capabilities(vec!["web_search".to_string(), "reading".to_string()])
        .with_tools(vec!["search".to_string(), "browser".to_string()]));

        self.register(SpecialistRole::new(
            "planner".to_string(),
            "Planner".to_string(),
            "Strategic planning and task breakdown".to_string(),
            "You are a planning specialist. Your task is to create detailed, actionable plans for complex projects. Break down work into clear steps, identify dependencies, and estimate effort.".to_string(),
        )
        .with_capabilities(vec!["planning".to_string(), "analysis".to_string()]));

        self.register(SpecialistRole::new(
            "architect".to_string(),
            "Architect".to_string(),
            "System architecture and design".to_string(),
            "You are an architecture specialist. Your task is to design robust, scalable systems. Consider trade-offs, identify patterns, and create clear architectural diagrams and explanations.".to_string(),
        )
        .with_capabilities(vec!["architecture".to_string(), "design".to_string()]));

        self.register(SpecialistRole::new(
            "coder".to_string(),
            "Coder".to_string(),
            "Code implementation and modification".to_string(),
            "You are a coding specialist. Your task is to write clean, efficient, well-documented code. Follow best practices, handle errors appropriately, and ensure code is maintainable.".to_string(),
        )
        .with_capabilities(vec!["coding".to_string(), "debugging".to_string()])
        .with_tools(vec!["filesystem".to_string(), "shell".to_string(), "grep".to_string()]));

        self.register(SpecialistRole::new(
            "refactorer".to_string(),
            "Refactorer".to_string(),
            "Code refactoring and improvement".to_string(),
            "You are a refactoring specialist. Your task is to improve code quality, reduce technical debt, and apply modern patterns while preserving functionality.".to_string(),
        )
        .with_capabilities(vec!["refactoring".to_string(), "code_analysis".to_string()]));

        self.register(SpecialistRole::new(
            "tester".to_string(),
            "Tester".to_string(),
            "Testing and quality assurance".to_string(),
            "You are a testing specialist. Your task is to design and implement comprehensive tests. Consider edge cases, integration scenarios, and ensure reliability.".to_string(),
        )
        .with_capabilities(vec!["testing".to_string(), "qa".to_string()])
        .with_tools(vec!["shell".to_string()]));

        self.register(SpecialistRole::new(
            "debugger".to_string(),
            "Debugger".to_string(),
            "Debugging and troubleshooting".to_string(),
            "You are a debugging specialist. Your task is to identify root causes of issues, reproduce problems systematically, and provide clear explanations of fixes.".to_string(),
        )
        .with_capabilities(vec!["debugging".to_string(), "analysis".to_string()])
        .with_tools(vec!["shell".to_string(), "grep".to_string()]));

        self.register(SpecialistRole::new(
            "reviewer".to_string(),
            "Reviewer".to_string(),
            "Code review and quality assessment".to_string(),
            "You are a code review specialist. Your task is to review code for correctness, style, best practices, and potential issues. Provide constructive feedback.".to_string(),
        )
        .with_capabilities(vec!["review".to_string(), "quality_assessment".to_string()]));

        self.register(SpecialistRole::new(
            "doc_writer".to_string(),
            "Documentation Writer".to_string(),
            "Technical documentation".to_string(),
            "You are a documentation specialist. Your task is to write clear, comprehensive technical documentation. Explain complex concepts simply and provide examples.".to_string(),
        )
        .with_capabilities(vec!["writing".to_string(), "documentation".to_string()]));

        self.register(SpecialistRole::new(
            "release_manager".to_string(),
            "Release Manager".to_string(),
            "Release management and deployment".to_string(),
            "You are a release management specialist. Your task is to manage software releases, coordinate deployments, and ensure smooth rollouts.".to_string(),
        )
        .with_capabilities(vec!["release".to_string(), "deployment".to_string()]));

        self.register(SpecialistRole::new(
            "ci_fixer".to_string(),
            "CI Fixer".to_string(),
            "CI/CD troubleshooting and fixes".to_string(),
            "You are a CI/CD specialist. Your task is to fix build failures, optimize pipelines, and ensure reliable automated processes.".to_string(),
        )
        .with_capabilities(vec!["ci_cd".to_string(), "automation".to_string()])
        .with_tools(vec!["shell".to_string()]));

        self.register(SpecialistRole::new(
            "dependency_auditor".to_string(),
            "Dependency Auditor".to_string(),
            "Dependency analysis and security".to_string(),
            "You are a dependency specialist. Your task is to audit dependencies for security vulnerabilities, outdated packages, and licensing issues.".to_string(),
        )
        .with_capabilities(vec!["security".to_string(), "dependency_management".to_string()]));

        self.register(SpecialistRole::new(
            "performance_investigator".to_string(),
            "Performance Investigator".to_string(),
            "Performance analysis and optimization".to_string(),
            "You are a performance specialist. Your task is to identify bottlenecks, analyze metrics, and suggest optimizations for performance improvements.".to_string(),
        )
        .with_capabilities(vec!["performance".to_string(), "profiling".to_string()]));

        self.register(SpecialistRole::new(
            "security_reviewer".to_string(),
            "Security Reviewer".to_string(),
            "Security analysis and best practices".to_string(),
            "You are a security specialist. Your task is to identify vulnerabilities, apply security best practices, and ensure code follows security guidelines.".to_string(),
        )
        .with_capabilities(vec!["security".to_string(), "audit".to_string()]));

        self.register(SpecialistRole::new(
            "browser_debugger".to_string(),
            "Browser Debugger".to_string(),
            "Web browser debugging and inspection".to_string(),
            "You are a browser debugging specialist. Your task is to debug web applications, inspect DOM, analyze network requests, and test in browsers.".to_string(),
        )
        .with_capabilities(vec!["browser".to_string(), "web_debugging".to_string()])
        .with_tools(vec!["browser".to_string()]));

        self.register(SpecialistRole::new(
            "issue_triager".to_string(),
            "Issue Triager".to_string(),
            "Issue analysis and prioritization".to_string(),
            "You are an issue triage specialist. Your task is to analyze bug reports, categorize issues, estimate severity, and prioritize fixes.".to_string(),
        )
        .with_capabilities(vec!["analysis".to_string(), "prioritization".to_string()]));

        self.register(SpecialistRole::new(
            "repo_analyst".to_string(),
            "Repository Analyst".to_string(),
            "Repository structure and codebase analysis".to_string(),
            "You are a repository analysis specialist. Your task is to understand codebase structure, identify patterns, and provide insights about project organization.".to_string(),
        )
        .with_capabilities(vec!["analysis".to_string(), "codebase_understanding".to_string()])
        .with_tools(vec!["filesystem".to_string(), "grep".to_string()]));

        self.register(SpecialistRole::new(
            "shell_executor".to_string(),
            "Shell Executor".to_string(),
            "Shell command execution and automation".to_string(),
            "You are a shell automation specialist. Your task is to execute shell commands efficiently, chain operations, and automate workflows.".to_string(),
        )
        .with_capabilities(vec!["shell".to_string(), "automation".to_string()])
        .with_tools(vec!["shell".to_string()]));

        self.register(SpecialistRole::new(
            "git_operator".to_string(),
            "Git Operator".to_string(),
            "Git operations and version control".to_string(),
            "You are a Git specialist. Your task is to perform Git operations, manage branches, resolve conflicts, and handle version control tasks.".to_string(),
        )
        .with_capabilities(vec!["git".to_string(), "version_control".to_string()])
        .with_tools(vec!["git".to_string()]));

        self.register(SpecialistRole::new(
            "ux_critic".to_string(),
            "UX Critic".to_string(),
            "User experience evaluation".to_string(),
            "You are a UX specialist. Your task is to evaluate user experience, identify pain points, and suggest improvements for better usability.".to_string(),
        )
        .with_capabilities(vec!["ux".to_string(), "user_experience".to_string()]));

        self.register(SpecialistRole::new(
            "prompt_optimizer".to_string(),
            "Prompt Optimizer".to_string(),
            "Prompt engineering and optimization".to_string(),
            "You are a prompt engineering specialist. Your task is to optimize prompts for better results, reduce ambiguity, and improve AI interactions.".to_string(),
        )
        .with_capabilities(vec!["prompt_engineering".to_string(), "optimization".to_string()]));

        self.register(SpecialistRole::new(
            "migration_planner".to_string(),
            "Migration Planner".to_string(),
            "Migration planning and execution".to_string(),
            "You are a migration specialist. Your task is to plan and execute migrations between frameworks, databases, or platforms with minimal disruption.".to_string(),
        )
        .with_capabilities(vec!["migration".to_string(), "planning".to_string()]));

        self.register(SpecialistRole::new(
            "api_designer".to_string(),
            "API Designer".to_string(),
            "API design and documentation".to_string(),
            "You are an API design specialist. Your task is to design clean, intuitive APIs, write documentation, and ensure good API practices.".to_string(),
        )
        .with_capabilities(vec!["api_design".to_string(), "rest".to_string()]));

        self.register(SpecialistRole::new(
            "database_specialist".to_string(),
            "Database Specialist".to_string(),
            "Database design and optimization".to_string(),
            "You are a database specialist. Your task is to design efficient schemas, optimize queries, and ensure data integrity and performance.".to_string(),
        )
        .with_capabilities(vec!["database".to_string(), "sql".to_string()]));

        self.register(SpecialistRole::new(
            "devops_engineer".to_string(),
            "DevOps Engineer".to_string(),
            "DevOps and infrastructure".to_string(),
            "You are a DevOps specialist. Your task is to manage infrastructure, set up CI/CD pipelines, and ensure reliable deployments.".to_string(),
        )
        .with_capabilities(vec!["devops".to_string(), "infrastructure".to_string()]));

        self.register(SpecialistRole::new(
            "frontend_engineer".to_string(),
            "Frontend Engineer".to_string(),
            "Frontend development and UI".to_string(),
            "You are a frontend specialist. Your task is to build responsive, accessible user interfaces with good UX and performance.".to_string(),
        )
        .with_capabilities(vec!["frontend".to_string(), "ui".to_string(), "css".to_string()]));

        self.register(SpecialistRole::new(
            "backend_engineer".to_string(),
            "Backend Engineer".to_string(),
            "Backend development and APIs".to_string(),
            "You are a backend specialist. Your task is to build robust backend services, APIs, and ensure good performance and scalability.".to_string(),
        )
        .with_capabilities(vec!["backend".to_string(), "api".to_string(), "server".to_string()]));

        self.register(SpecialistRole::new(
            "mobile_developer".to_string(),
            "Mobile Developer".to_string(),
            "Mobile app development".to_string(),
            "You are a mobile development specialist. Your task is to build mobile applications for iOS and Android with good UX and performance.".to_string(),
        )
        .with_capabilities(vec!["mobile".to_string(), "ios".to_string(), "android".to_string()]));

        self.register(SpecialistRole::new(
            "cloud_architect".to_string(),
            "Cloud Architect".to_string(),
            "Cloud architecture and deployment".to_string(),
            "You are a cloud architecture specialist. Your task is to design cloud-native solutions, optimize costs, and ensure reliability and scalability.".to_string(),
        )
        .with_capabilities(vec!["cloud".to_string(), "aws".to_string(), "gcp".to_string(), "azure".to_string()]));

        self.register(SpecialistRole::new(
            "data_engineer".to_string(),
            "Data Engineer".to_string(),
            "Data pipelines and ETL".to_string(),
            "You are a data engineering specialist. Your task is to build data pipelines, ETL processes, and ensure data quality and reliability.".to_string(),
        )
        .with_capabilities(vec!["data".to_string(), "etl".to_string(), "pipelines".to_string()]));

        self.register(SpecialistRole::new(
            "ml_engineer".to_string(),
            "ML Engineer".to_string(),
            "Machine learning implementation".to_string(),
            "You are an ML engineering specialist. Your task is to implement ML models, build training pipelines, and deploy ML services.".to_string(),
        )
        .with_capabilities(vec!["machine_learning".to_string(), "ml".to_string()]));

        self.register(SpecialistRole::new(
            "security_auditor".to_string(),
            "Security Auditor".to_string(),
            "Security auditing and penetration testing".to_string(),
            "You are a security audit specialist. Your task is to perform security audits, identify vulnerabilities, and recommend security improvements.".to_string(),
        )
        .with_capabilities(vec!["security".to_string(), "penetration_testing".to_string(), "audit".to_string()]));

        self.register(SpecialistRole::new(
            "compliance_specialist".to_string(),
            "Compliance Specialist".to_string(),
            "Compliance and regulatory requirements".to_string(),
            "You are a compliance specialist. Your task is to ensure software meets regulatory requirements, follows standards, and maintains compliance.".to_string(),
        )
        .with_capabilities(vec!["compliance".to_string(), "regulatory".to_string(), "standards".to_string()]));

        self.register(SpecialistRole::new(
            "accessibility_specialist".to_string(),
            "Accessibility Specialist".to_string(),
            "Accessibility and inclusive design".to_string(),
            "You are an accessibility specialist. Your task is to ensure software is accessible to all users, following WCAG guidelines and best practices.".to_string(),
        )
        .with_capabilities(vec!["accessibility".to_string(), "a11y".to_string(), "inclusive_design".to_string()]));

        self.register(SpecialistRole::new(
            "i18n_specialist".to_string(),
            "Internationalization Specialist".to_string(),
            "Internationalization and localization".to_string(),
            "You are an i18n specialist. Your task is to ensure software supports multiple languages and regions, with proper localization.".to_string(),
        )
        .with_capabilities(vec!["i18n".to_string(), "l10n".to_string(), "localization".to_string()]));

        self.register(SpecialistRole::new(
            "monitoring_specialist".to_string(),
            "Monitoring Specialist".to_string(),
            "Monitoring and observability".to_string(),
            "You are a monitoring specialist. Your task is to set up monitoring, alerts, and ensure observability of systems and applications.".to_string(),
        )
        .with_capabilities(vec!["monitoring".to_string(), "observability".to_string(), "metrics".to_string()]));

        self.register(SpecialistRole::new(
            "log_analyst".to_string(),
            "Log Analyst".to_string(),
            "Log analysis and troubleshooting".to_string(),
            "You are a log analysis specialist. Your task is to analyze logs, identify issues, and provide insights from system logs.".to_string(),
        )
        .with_capabilities(vec!["logs".to_string(), "analysis".to_string(), "troubleshooting".to_string()]));

        self.register(SpecialistRole::new(
            "network_specialist".to_string(),
            "Network Specialist".to_string(),
            "Network configuration and debugging".to_string(),
            "You are a network specialist. Your task is to configure networks, debug connectivity issues, and ensure network security and performance.".to_string(),
        )
        .with_capabilities(vec!["networking".to_string(), "tcp_ip".to_string(), "firewall".to_string()]));

        self.register(SpecialistRole::new(
            "container_specialist".to_string(),
            "Container Specialist".to_string(),
            "Container orchestration and deployment".to_string(),
            "You are a container specialist. Your task is to manage Docker containers, Kubernetes deployments, and ensure container best practices.".to_string(),
        )
        .with_capabilities(vec!["docker".to_string(), "kubernetes".to_string(), "containers".to_string()]));

        self.register(SpecialistRole::new(
            "caching_specialist".to_string(),
            "Caching Specialist".to_string(),
            "Caching strategies and implementation".to_string(),
            "You are a caching specialist. Your task is to design and implement caching strategies to improve performance and reduce load.".to_string(),
        )
        .with_capabilities(vec!["caching".to_string(), "redis".to_string(), "memcached".to_string()]));

        self.register(SpecialistRole::new(
            "async_specialist".to_string(),
            "Async Specialist".to_string(),
            "Async/await and concurrency".to_string(),
            "You are an async programming specialist. Your task is to write efficient async code, handle concurrency properly, and avoid common async pitfalls.".to_string(),
        )
        .with_capabilities(vec!["async".to_string(), "concurrency".to_string(), "futures".to_string()]));

        self.register(SpecialistRole::new(
            "error_handling_specialist".to_string(),
            "Error Handling Specialist".to_string(),
            "Error handling and resilience".to_string(),
            "You are an error handling specialist. Your task is to design robust error handling, implement retries, and ensure system resilience.".to_string(),
        )
        .with_capabilities(vec!["error_handling".to_string(), "resilience".to_string(), "fault_tolerance".to_string()]));

        self.register(SpecialistRole::new(
            "testing_strategist".to_string(),
            "Testing Strategist".to_string(),
            "Testing strategy and coverage".to_string(),
            "You are a testing strategy specialist. Your task is to design comprehensive testing strategies, ensure coverage, and balance test types.".to_string(),
        )
        .with_capabilities(vec!["testing".to_string(), "strategy".to_string(), "coverage".to_string()]));

        self.register(SpecialistRole::new(
            "benchmark_specialist".to_string(),
            "Benchmark Specialist".to_string(),
            "Performance benchmarking and profiling".to_string(),
            "You are a benchmark specialist. Your task is to run performance benchmarks, profile code, and identify optimization opportunities.".to_string(),
        )
        .with_capabilities(vec!["benchmarking".to_string(), "profiling".to_string(), "performance".to_string()]));

        self.register(SpecialistRole::new(
            "memory_specialist".to_string(),
            "Memory Specialist".to_string(),
            "Memory usage and optimization".to_string(),
            "You are a memory specialist. Your task is to analyze memory usage, identify leaks, and optimize memory consumption.".to_string(),
        )
        .with_capabilities(vec!["memory".to_string(), "optimization".to_string(), "profiling".to_string()]));

        self.register(SpecialistRole::new(
            "crypto_specialist".to_string(),
            "Cryptography Specialist".to_string(),
            "Cryptography and security implementations".to_string(),
            "You are a cryptography specialist. Your task is to implement secure cryptographic operations, follow best practices, and avoid common crypto pitfalls.".to_string(),
        )
        .with_capabilities(vec!["cryptography".to_string(), "security".to_string(), "encryption".to_string()]));

        self.register(SpecialistRole::new(
            "json_specialist".to_string(),
            "JSON Specialist".to_string(),
            "JSON processing and parsing".to_string(),
            "You are a JSON processing specialist. Your task is to handle JSON data efficiently, parse complex structures, and implement JSON schemas.".to_string(),
        )
        .with_capabilities(vec!["json".to_string(), "parsing".to_string(), "serialization".to_string()]));

        self.register(SpecialistRole::new(
            "xml_specialist".to_string(),
            "XML Specialist".to_string(),
            "XML processing and parsing".to_string(),
            "You are an XML processing specialist. Your task is to handle XML data efficiently, parse complex structures, and implement XML schemas.".to_string(),
        )
        .with_capabilities(vec!["xml".to_string(), "parsing".to_string(), "serialization".to_string()]));

        self.register(SpecialistRole::new(
            "regex_specialist".to_string(),
            "Regex Specialist".to_string(),
            "Regular expressions and pattern matching".to_string(),
            "You are a regex specialist. Your task is to write efficient regular expressions, optimize patterns, and solve complex matching problems.".to_string(),
        )
        .with_capabilities(vec!["regex".to_string(), "pattern_matching".to_string(), "text_processing".to_string()]));

        self.register(SpecialistRole::new(
            "cli_specialist".to_string(),
            "CLI Specialist".to_string(),
            "Command-line interface design".to_string(),
            "You are a CLI specialist. Your task is to design intuitive command-line interfaces, follow CLI conventions, and ensure good UX.".to_string(),
        )
        .with_capabilities(vec!["cli".to_string(), "terminal".to_string(), "ux".to_string()]));

        self.register(SpecialistRole::new(
            "default".to_string(),
            "Default".to_string(),
            "General-purpose coding and assistance".to_string(),
            "You are Fever Code, a senior full-stack developer and coding assistant. You help users build, debug, and improve software projects. You're thorough, practical, and follow best practices.".to_string(),
        )
        .with_capabilities(vec!["coding".to_string(), "debugging".to_string(), "analysis".to_string()])
        .with_tools(vec!["filesystem".to_string(), "shell".to_string(), "grep".to_string(), "git".to_string()]));
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

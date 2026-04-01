/// Generated file descriptor used to represent scaffold outputs.
use crate::profile::ProjectProfile;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

pub struct ScaffoldGenerator {
    pub profile: ProjectProfile,
}

impl ScaffoldGenerator {
    pub fn new(profile: ProjectProfile) -> Self {
        Self { profile }
    }

    pub fn generate_all(&self) -> Result<Vec<GeneratedFile>, String> {
        let mut files = Vec::new();
        if let Some(dep) = self.generate_deployment_config() {
            files.push(dep);
        }
        if let Some(docker) = self.generate_dockerfile() {
            files.push(docker);
        }
        for f in self.generate_ci_cd() {
            files.push(f);
        }
        files.push(self.generate_readme());
        files.push(self.generate_env_example());
        Ok(files)
    }

    pub fn generate_deployment_config(&self) -> Option<GeneratedFile> {
        match self.profile.hosting_platform.as_str() {
            "Railway" => Some(GeneratedFile {
                path: "railway.toml".to_string(),
                content: format!("[project]\nname = \"{}\"\n", self.profile.project_name),
            }),
            "Render" => Some(GeneratedFile {
                path: "render.yaml".to_string(),
                content: format!("# render config for {}\n", self.profile.project_name),
            }),
            "Fly.io" => Some(GeneratedFile {
                path: "fly.toml".to_string(),
                content: format!("app = \"{}\"\n", self.profile.project_name),
            }),
            _ => None,
        }
    }

    pub fn generate_dockerfile(&self) -> Option<GeneratedFile> {
        let lang = self.profile.primary_language.to_lowercase();
        let content = match lang.as_str() {
            "rust" => {
                "FROM rust:1.70-slim AS builder\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nFROM debian:buster-slim\nCOPY --from=builder /app/target/release/fever-app /usr/local/bin/fever-app\nCMD [\"fever-app\"]\n"
            }
            "node" | "javascript" | "typescript" => {
                "FROM node:20-alpine AS builder\nWORKDIR /app\nCOPY package*.json ./\nRUN npm install --omit=dev\nCOPY . .\nRUN npm run build --silent\nFROM node:20-alpine\nCOPY --from=builder /app/dist /app/dist\nCMD [\"node\", \"dist/index.js\"]\n"
            }
            "python" => {
                "FROM python:3.12-slim AS builder\nWORKDIR /app\nCOPY . .\nRUN pip install -r requirements.txt\nRUN python setup.py install\nFROM python:3.12-slim\nCOPY --from=builder /usr/local/lib/python3.12/site-packages /usr/local/lib/python3.12/site-packages\nCMD [\"python\", \"app.py\"]\n"
            }
            _ => {
                "FROM scratch\n"
            }
        };
        Some(GeneratedFile {
            path: "Dockerfile".to_string(),
            content: content.to_string(),
        })
    }

    pub fn generate_ci_cd(&self) -> Vec<GeneratedFile> {
        if self.profile.cicd_needed.to_lowercase() == "none" {
            return vec![];
        }
        vec![
            GeneratedFile {
                path: ".github/workflows/ci.yml".to_string(),
                content: String::from("name: ci\nnpm: false\n"),
            },
            GeneratedFile {
                path: ".github/workflows/deploy.yml".to_string(),
                content: String::from("name: deploy\non: push\n"),
            },
        ]
    }

    pub fn generate_readme(&self) -> GeneratedFile {
        GeneratedFile {
            path: "README.md".to_string(),
            content: format!(
                "# {}\n\n{}\n",
                self.profile.project_name, self.profile.description
            ),
        }
    }

    pub fn generate_env_example(&self) -> GeneratedFile {
        let mut lines = vec![];
        for v in &self.profile.env_vars {
            lines.push(format!("{}=<value>", v));
        }
        GeneratedFile {
            path: ".env.example".to_string(),
            content: lines.join("\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ProjectProfile;
    #[test]
    fn deployment_config_railway_and_fly() {
        let mut p = ProjectProfile::default();
        p.project_name = "demo".to_string();
        p.description = "desc".to_string();
        p.hosting_platform = "Railway".to_string();
        let deployment = ScaffoldGenerator::new(p).generate_deployment_config();
        assert!(deployment.is_some());
        let p2 = {
            let mut p = ProjectProfile::default();
            p.hosting_platform = "Fly.io".to_string();
            p
        };
        let deployment2 = ScaffoldGenerator::new(p2).generate_deployment_config();
        assert!(deployment2.is_some());
    }

    #[test]
    fn dockerfile_generation_known_langs() {
        let mut p = ProjectProfile::default();
        p.primary_language = "Rust".to_string();
        let docker = ScaffoldGenerator::new(p).generate_dockerfile().unwrap();
        assert!(docker.path == "Dockerfile");
        assert!(docker.content.contains("FROM rust"));
    }

    #[test]
    fn ci_cd_generation_none_behaviour() {
        let mut p = ProjectProfile::default();
        p.cicd_needed = "None".to_string();
        let di = ScaffoldGenerator::new(p).generate_ci_cd();
        assert!(di.is_empty());
    }

    #[test]
    fn env_example_generation() {
        let mut p = ProjectProfile::default();
        p.env_vars = vec!["API_KEY".to_string(), "DEBUG".to_string()];
        let env = ScaffoldGenerator::new(p).generate_env_example();
        assert!(env.content.contains("API_KEY=<value>"));
        assert!(env.content.contains("DEBUG=<value>"));
    }
}

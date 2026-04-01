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
        let name = &self.profile.project_name;
        match self.profile.hosting_platform.as_str() {
            "Railway" => Some(GeneratedFile {
                path: "railway.toml".to_string(),
                content: format!("[project]\nname = \"{name}\"\n"),
            }),
            "Render" => Some(GeneratedFile {
                path: "render.yaml".to_string(),
                content: format!("# render config for {name}\n"),
            }),
            "Fly.io" => Some(GeneratedFile {
                path: "fly.toml".to_string(),
                content: format!("app = \"{name}\"\n"),
            }),
            "AWS" => Some(GeneratedFile {
                path: "aws.tf".to_string(),
                content: format!(
                    "provider \"aws\" {{ region = \"us-east-1\" }}\n\
                     \n\
                     resource \"aws_instance\" \"app\" {{\n\
                       ami           = \"ami-0c94855ba95c71c99\"\n\
                       instance_type = \"t2.micro\"\n\
                       tags          = {{ Name = \"{name}-instance\" }}\n\
                     }}\n\
                     \n\
                     resource \"aws_lb\" \"alb\" {{\n\
                       name               = \"{name}-alb\"\n\
                       internal           = false\n\
                       load_balancer_type = \"application\"\n\
                       subnets            = [\"subnet-00000000\"]\n\
                     }}\n"
                ),
            }),
            "GCP" => Some(GeneratedFile {
                path: "cloudbuild.yaml".to_string(),
                content: format!(
                    "steps:\n\
                     - name: 'gcr.io/cloud-builders/docker'\n\
                       args: ['build', '-t', 'gcr.io/$PROJECT_ID/{name}-image', '.']\n\
                     - name: 'gcr.io/cloud-builders/gcloud'\n\
                       args: ['run', 'deploy', '{name}-service', '--image', 'gcr.io/$PROJECT_ID/{name}-image', '--region', 'us-central1', '--platform', 'managed']\n"
                ),
            }),
            "DigitalOcean" => Some(GeneratedFile {
                path: "do-app.yaml".to_string(),
                content: format!(
                    "name: {name}\n\
                     services:\n\
                       - type: web\n\
                         dockerfile_path: Dockerfile\n\
                         http_port: 8080\n"
                ),
            }),
            "VPS" => Some(GeneratedFile {
                path: "deploy.sh".to_string(),
                content: format!(
                    "#!/bin/bash\n\
                     set -e\n\
                     APP_NAME='{name}'\n\
                     WORKDIR='/opt/{name}'\n\
                     mkdir -p \"$WORKDIR\"\n\
                     cd \"$WORKDIR\"\n\
                     cargo build --release\n\
                     sudo systemctl stop {name}.service 2>/dev/null || true\n\
                     sudo cp target/release/{name} /usr/local/bin/{name}\n\
                     sudo systemctl start {name}.service\n"
                ),
            }),
            "Docker" => Some(GeneratedFile {
                path: "docker-compose.yml".to_string(),
                content: format!(
                    "version: \"3.8\"\n\
                     services:\n\
                       app:\n\
                         build: .\n\
                         container_name: {name}-app\n\
                         ports:\n\
                           - \"8080:8080\"\n"
                ),
            }),
            _ => None,
        }
    }

    pub fn generate_dockerfile(&self) -> Option<GeneratedFile> {
        let lang = self.profile.primary_language.to_lowercase();
        let content = match lang.as_str() {
            "rust" => {
                "FROM rust:1.70-slim AS builder\n\
                 WORKDIR /app\n\
                 COPY . .\n\
                 RUN cargo build --release\n\
                 FROM debian:buster-slim\n\
                 COPY --from=builder /app/target/release/fever-app /usr/local/bin/fever-app\n\
                 CMD [\"fever-app\"]\n"
            }
            "node" | "javascript" | "typescript" => {
                "FROM node:20-alpine AS builder\n\
                 WORKDIR /app\n\
                 COPY package*.json ./\n\
                 RUN npm install --omit=dev\n\
                 COPY . .\n\
                 RUN npm run build --silent\n\
                 FROM node:20-alpine\n\
                 COPY --from=builder /app/dist /app/dist\n\
                 CMD [\"node\", \"dist/index.js\"]\n"
            }
            "python" => {
                "FROM python:3.12-slim AS builder\n\
                 WORKDIR /app\n\
                 COPY . .\n\
                 RUN pip install -r requirements.txt\n\
                 RUN python setup.py install\n\
                 FROM python:3.12-slim\n\
                 COPY --from=builder /usr/local/lib/python3.12/site-packages /usr/local/lib/python3.12/site-packages\n\
                 CMD [\"python\", \"app.py\"]\n"
            }
            _ => "FROM scratch\n",
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

        let name = &self.profile.project_name;
        let lang = self.profile.primary_language.to_lowercase();

        let ci_yaml = match lang.as_str() {
            "rust" => "name: ci\n\
                 on: [push, pull_request]\n\
                 \n\
                 jobs:\n\
                   build:\n\
                     runs-on: ubuntu-latest\n\
                     steps:\n\
                       - uses: actions/checkout@v4\n\
                       - name: Install Rust\n\
                         uses: dtolnay/rust-toolchain@stable\n\
                       - name: Build\n\
                         run: cargo build --workspace\n\
                       - name: Test\n\
                         run: cargo test --workspace\n\
                       - name: Clippy\n\
                         run: cargo clippy --workspace\n\
                       - name: Format check\n\
                         run: cargo fmt --all -- --check\n"
                .to_string(),
            "node" | "javascript" | "typescript" => "name: ci\n\
                 on: [push, pull_request]\n\
                 \n\
                 jobs:\n\
                   build:\n\
                     runs-on: ubuntu-latest\n\
                     steps:\n\
                       - uses: actions/checkout@v4\n\
                       - name: Setup Node\n\
                         uses: actions/setup-node@v4\n\
                         with:\n\
                           node-version: '20'\n\
                       - name: Install dependencies\n\
                         run: npm ci\n\
                       - name: Build\n\
                         run: npm run build\n\
                       - name: Test\n\
                         run: npm test\n\
                       - name: Lint\n\
                         run: npm run lint\n"
                .to_string(),
            "python" => "name: ci\n\
                 on: [push, pull_request]\n\
                 \n\
                 jobs:\n\
                   build:\n\
                     runs-on: ubuntu-latest\n\
                     steps:\n\
                       - uses: actions/checkout@v4\n\
                       - name: Setup Python\n\
                         uses: actions/setup-python@v5\n\
                         with:\n\
                           python-version: '3.12'\n\
                       - name: Install dependencies\n\
                         run: pip install -r requirements.txt\n\
                       - name: Test\n\
                         run: pytest\n"
                .to_string(),
            _ => format!(
                "name: ci\n\
                 on: [push, pull_request]\n\
                 \n\
                 jobs:\n\
                   build:\n\
                     runs-on: ubuntu-latest\n\
                     steps:\n\
                       - uses: actions/checkout@v4\n\
                       - name: Build\n\
                         run: echo 'Build {name}'\n\
                       - name: Test\n\
                         run: echo 'Test {name}'\n"
            ),
        };

        let deploy_yaml = format!(
            "name: deploy\n\
             on:\n\
               push:\n\
                 branches: [main]\n\
             \n\
             jobs:\n\
               deploy:\n\
                 runs-on: ubuntu-latest\n\
                 needs: [ci]\n\
                 steps:\n\
                   - uses: actions/checkout@v4\n\
                   - name: Deploy\n\
                     run: echo 'Deploy {name} - fill in deployment steps here'\n"
        );

        vec![
            GeneratedFile {
                path: ".github/workflows/ci.yml".to_string(),
                content: ci_yaml,
            },
            GeneratedFile {
                path: ".github/workflows/deploy.yml".to_string(),
                content: deploy_yaml,
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
        let p = ProjectProfile {
            project_name: "demo".to_string(),
            description: "desc".to_string(),
            hosting_platform: "Railway".to_string(),
            ..Default::default()
        };
        let deployment = ScaffoldGenerator::new(p).generate_deployment_config();
        assert!(deployment.is_some());
        let p2 = ProjectProfile {
            hosting_platform: "Fly.io".to_string(),
            ..Default::default()
        };
        let deployment2 = ScaffoldGenerator::new(p2).generate_deployment_config();
        assert!(deployment2.is_some());
    }

    #[test]
    fn dockerfile_generation_known_langs() {
        let p = ProjectProfile {
            primary_language: "Rust".to_string(),
            ..Default::default()
        };
        let docker = ScaffoldGenerator::new(p).generate_dockerfile().unwrap();
        assert!(docker.path == "Dockerfile");
        assert!(docker.content.contains("FROM rust"));
    }

    #[test]
    fn ci_cd_generation_none_behaviour() {
        let p = ProjectProfile {
            cicd_needed: "None".to_string(),
            ..Default::default()
        };
        let di = ScaffoldGenerator::new(p).generate_ci_cd();
        assert!(di.is_empty());
    }

    #[test]
    fn env_example_generation() {
        let p = ProjectProfile {
            env_vars: vec!["API_KEY".to_string(), "DEBUG".to_string()],
            ..Default::default()
        };
        let env = ScaffoldGenerator::new(p).generate_env_example();
        assert!(env.content.contains("API_KEY=<value>"));
        assert!(env.content.contains("DEBUG=<value>"));
    }

    #[test]
    fn deployment_config_aws() {
        let p = ProjectProfile {
            project_name: "myapp".to_string(),
            hosting_platform: "AWS".to_string(),
            ..Default::default()
        };
        let dep = ScaffoldGenerator::new(p)
            .generate_deployment_config()
            .unwrap();
        assert_eq!(dep.path, "aws.tf");
        assert!(dep.content.contains("aws_instance"));
        assert!(dep.content.contains("myapp"));
    }

    #[test]
    fn deployment_config_gcp() {
        let p = ProjectProfile {
            project_name: "myapp".to_string(),
            hosting_platform: "GCP".to_string(),
            ..Default::default()
        };
        let dep = ScaffoldGenerator::new(p)
            .generate_deployment_config()
            .unwrap();
        assert_eq!(dep.path, "cloudbuild.yaml");
        assert!(dep.content.contains("cloud-builders"));
        assert!(dep.content.contains("myapp"));
    }

    #[test]
    fn deployment_config_digitalocean() {
        let p = ProjectProfile {
            project_name: "myapp".to_string(),
            hosting_platform: "DigitalOcean".to_string(),
            ..Default::default()
        };
        let dep = ScaffoldGenerator::new(p)
            .generate_deployment_config()
            .unwrap();
        assert_eq!(dep.path, "do-app.yaml");
        assert!(dep.content.contains("services:"));
        assert!(dep.content.contains("myapp"));
    }

    #[test]
    fn deployment_config_vps() {
        let p = ProjectProfile {
            project_name: "myapp".to_string(),
            hosting_platform: "VPS".to_string(),
            ..Default::default()
        };
        let dep = ScaffoldGenerator::new(p)
            .generate_deployment_config()
            .unwrap();
        assert_eq!(dep.path, "deploy.sh");
        assert!(dep.content.contains("#!/bin/bash"));
        assert!(dep.content.contains("myapp"));
    }

    #[test]
    fn deployment_config_docker() {
        let p = ProjectProfile {
            project_name: "myapp".to_string(),
            hosting_platform: "Docker".to_string(),
            ..Default::default()
        };
        let dep = ScaffoldGenerator::new(p)
            .generate_deployment_config()
            .unwrap();
        assert_eq!(dep.path, "docker-compose.yml");
        assert!(dep.content.contains("services:"));
        assert!(dep.content.contains("myapp"));
    }

    #[test]
    fn ci_cd_yaml_contains_real_structure() {
        let p = ProjectProfile {
            project_name: "test".to_string(),
            primary_language: "Rust".to_string(),
            cicd_needed: "GitHub Actions".to_string(),
            ..Default::default()
        };
        let files = ScaffoldGenerator::new(p).generate_ci_cd();
        let ci = &files[0];
        assert_eq!(ci.path, ".github/workflows/ci.yml");
        assert!(ci.content.contains("runs-on:"));
        assert!(ci.content.contains("steps:"));
        assert!(ci.content.contains("actions/checkout"));
        assert!(ci.content.contains("cargo build"));
        assert!(ci.content.contains("cargo test"));
    }
}

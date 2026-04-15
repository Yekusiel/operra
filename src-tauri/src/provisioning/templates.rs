use super::ProvisioningConfig;

/// Generate a complete, tested setup.sh script from a template.
/// No AI involved -- these are deterministic scripts with variable substitution.
pub fn generate_setup_script(config: &ProvisioningConfig) -> String {
    let mut script = String::new();

    // Header
    script.push_str(&header(config));

    // System setup (common to all templates)
    script.push_str(&system_setup(config));

    // Deploy key setup (for GitHub projects)
    if config.source_type == "github" {
        script.push_str(&deploy_key_setup(config));
    }

    // Clone repository
    script.push_str(&clone_repo(config));

    // Stack-specific provisioning
    match config.stack_type.as_str() {
        "nextjs-docker-compose" => script.push_str(&nextjs_docker_compose(config)),
        "nextjs-standalone" => script.push_str(&nextjs_standalone(config)),
        "node-api-docker" => script.push_str(&node_api_docker(config)),
        "node-api-standalone" => script.push_str(&node_api_standalone(config)),
        "static-site" => script.push_str(&static_site(config)),
        "docker-compose-full" => script.push_str(&docker_compose_full(config)),
        _ => script.push_str(&nextjs_standalone(config)), // Safe fallback
    }

    // Caddy reverse proxy (common to all except static-site which uses Caddy directly)
    if config.stack_type != "static-site" {
        script.push_str(&caddy_setup(config));
    }

    // Deploy helper script
    script.push_str(&deploy_helper(config));

    // Footer
    script.push_str(&format!("\necho \"=== {} setup complete at $(date) ===\"\n", config.project_name));

    script
}

fn header(config: &ProvisioningConfig) -> String {
    format!(r#"#!/bin/bash
set -euo pipefail
exec > /var/log/user-data.log 2>&1

echo "=== {project} setup starting at $(date) ==="
echo "Stack type: {stack}"
echo "Region: {region}"

"#,
        project = config.project_name,
        stack = config.stack_type,
        region = config.aws_region,
    )
}

fn system_setup(config: &ProvisioningConfig) -> String {
    format!(r#"# ---------- System setup ----------
export DEBIAN_FRONTEND=noninteractive

# Create swap space (safety net for builds on small instances)
if [ ! -f /swapfile ]; then
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo "Swap enabled: 2GB"
fi

# Detect OS
if command -v apt-get &>/dev/null; then
    PKG_MANAGER="apt"
    apt-get update -y
    apt-get install -y curl git jq unzip
elif command -v dnf &>/dev/null; then
    PKG_MANAGER="dnf"
    dnf update -y
    dnf install -y curl git jq unzip
elif command -v yum &>/dev/null; then
    PKG_MANAGER="yum"
    yum update -y
    yum install -y curl git jq unzip
fi

# Install AWS CLI if not present
if ! command -v aws &>/dev/null; then
    curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-$(uname -m).zip" -o /tmp/awscli.zip
    unzip -q /tmp/awscli.zip -d /tmp
    /tmp/aws/install
    rm -rf /tmp/aws /tmp/awscli.zip
fi

# Install Docker
if ! command -v docker &>/dev/null; then
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
    usermod -aG docker {ssh_user}
fi

echo "System setup complete"

"#,
        ssh_user = config.ssh_user,
    )
}

fn deploy_key_setup(config: &ProvisioningConfig) -> String {
    format!(r#"# ---------- Deploy key setup ----------
mkdir -p /root/.ssh
chmod 700 /root/.ssh

aws ssm get-parameter \
    --name "{ssm_path}" \
    --with-decryption \
    --query "Parameter.Value" \
    --output text \
    --region "{region}" > /root/.ssh/deploy_key

chmod 600 /root/.ssh/deploy_key

cat > /root/.ssh/config <<'SSHCFG'
Host github.com
    IdentityFile /root/.ssh/deploy_key
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
SSHCFG
chmod 600 /root/.ssh/config

echo "Deploy key configured"

"#,
        ssm_path = config.deploy_key_ssm_path,
        region = config.aws_region,
    )
}

fn clone_repo(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    if config.source_type == "github" {
        format!(r#"# ---------- Clone repository ----------
APP_DIR="{app_dir}"

if [ -d "$APP_DIR" ]; then
    cd "$APP_DIR"
    git pull origin {branch}
else
    git clone --branch {branch} git@github.com:{repo}.git "$APP_DIR"
    cd "$APP_DIR"
fi

echo "Repository cloned to $APP_DIR"

"#,
            app_dir = app_dir,
            repo = config.github_repo,
            branch = config.github_branch,
        )
    } else {
        format!(r#"# ---------- App directory ----------
APP_DIR="{app_dir}"
mkdir -p "$APP_DIR"
cd "$APP_DIR"
echo "App directory ready (local project -- code must be pushed separately)"

"#,
            app_dir = app_dir,
        )
    }
}

fn nextjs_docker_compose(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Next.js + Docker Compose (DB only) ----------
# Docker Compose runs the database, Next.js runs natively with PM2

# Install Node.js {node_version}
if ! command -v node &>/dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_{node_version}.x | bash -
    apt-get install -y nodejs || dnf install -y nodejs
fi

# Install PM2
npm install -g pm2

# Generate secrets
DB_PASSWORD=$(openssl rand -hex 16)
NEXTAUTH_SECRET=$(openssl rand -hex 32)

# Create .env for the app
cat > "{app_dir}/.env" <<ENVFILE
DATABASE_URL=postgresql://{db_user}:${{DB_PASSWORD}}@localhost:5432/{db_name}
POSTGRES_USER={db_user}
POSTGRES_PASSWORD=${{DB_PASSWORD}}
POSTGRES_DB={db_name}
NODE_ENV=production
NEXTAUTH_SECRET=${{NEXTAUTH_SECRET}}
NEXTAUTH_URL={app_url}
ENVFILE
chmod 600 "{app_dir}/.env"

# Start database via Docker Compose
cd "{app_dir}"
docker compose up -d

# Wait for database to be ready
echo "Waiting for database..."
for i in $(seq 1 30); do
    if docker exec {project}-db pg_isready -U {db_user} 2>/dev/null; then
        echo "Database ready after $i seconds"
        break
    fi
    sleep 2
done

# Install Node.js dependencies
cd "{app_dir}"
npm ci --production=false

# Run Prisma migrations if present
if [ -f "prisma/schema.prisma" ]; then
    npx prisma generate
    npx prisma migrate deploy
    echo "Prisma migrations complete"
fi

# Build Next.js
npm run build

# Start with PM2
pm2 start npm --name "{project}" -- start
pm2 save
env PATH=$PATH:/usr/bin /usr/lib/node_modules/pm2/bin/pm2 startup systemd -u {ssh_user} --hp /home/{ssh_user} || true

echo "Next.js app started with PM2"

"#,
        node_version = config.node_version,
        app_dir = app_dir,
        db_user = config.db_user,
        db_name = config.db_name,
        project = config.project_name,
        ssh_user = config.ssh_user,
        app_url = if config.domain.is_empty() {
            format!("http://localhost:{}", config.app_port)
        } else {
            format!("https://{}", config.domain)
        },
    )
}

fn nextjs_standalone(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Next.js Standalone ----------
# Install Node.js {node_version}
if ! command -v node &>/dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_{node_version}.x | bash -
    apt-get install -y nodejs || dnf install -y nodejs
fi

# Install PM2
npm install -g pm2

# Create .env
NEXTAUTH_SECRET=$(openssl rand -hex 32)
cat > "{app_dir}/.env" <<ENVFILE
NODE_ENV=production
NEXTAUTH_SECRET=${{NEXTAUTH_SECRET}}
NEXTAUTH_URL={app_url}
ENVFILE
chmod 600 "{app_dir}/.env"

# Install dependencies and build
cd "{app_dir}"
npm ci --production=false
npm run build

# Start with PM2
pm2 start npm --name "{project}" -- start
pm2 save
env PATH=$PATH:/usr/bin /usr/lib/node_modules/pm2/bin/pm2 startup systemd -u {ssh_user} --hp /home/{ssh_user} || true

echo "Next.js app started with PM2"

"#,
        node_version = config.node_version,
        app_dir = app_dir,
        project = config.project_name,
        ssh_user = config.ssh_user,
        app_url = if config.domain.is_empty() {
            format!("http://localhost:{}", config.app_port)
        } else {
            format!("https://{}", config.domain)
        },
    )
}

fn node_api_docker(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Node.js API with Docker ----------
cd "{app_dir}"

# Build and run via Docker Compose if compose file exists
if [ -f "docker-compose.yml" ] || [ -f "compose.yml" ]; then
    docker compose up -d --build
    echo "App started via Docker Compose"
elif [ -f "Dockerfile" ]; then
    docker build -t {project} .
    docker run -d --name {project} --restart unless-stopped -p {port}:{port} {project}
    echo "App started via Docker"
fi

"#,
        app_dir = app_dir,
        project = config.project_name,
        port = config.app_port,
    )
}

fn node_api_standalone(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Node.js API Standalone ----------
# Install Node.js {node_version}
if ! command -v node &>/dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_{node_version}.x | bash -
    apt-get install -y nodejs || dnf install -y nodejs
fi

# Install PM2
npm install -g pm2

cd "{app_dir}"
npm ci --production

# Determine start command
if grep -q '"start"' package.json; then
    pm2 start npm --name "{project}" -- start
elif [ -f "server.js" ]; then
    pm2 start server.js --name "{project}"
elif [ -f "index.js" ]; then
    pm2 start index.js --name "{project}"
elif [ -f "app.js" ]; then
    pm2 start app.js --name "{project}"
fi

pm2 save
env PATH=$PATH:/usr/bin /usr/lib/node_modules/pm2/bin/pm2 startup systemd -u {ssh_user} --hp /home/{ssh_user} || true

echo "Node.js API started with PM2"

"#,
        node_version = config.node_version,
        app_dir = app_dir,
        project = config.project_name,
        ssh_user = config.ssh_user,
    )
}

fn static_site(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Static Site ----------
# Install Node.js for build
if ! command -v node &>/dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_{node_version}.x | bash -
    apt-get install -y nodejs || dnf install -y nodejs
fi

cd "{app_dir}"
npm ci
npm run build

# Serve static files with Caddy
# Detect build output directory
BUILD_DIR="{app_dir}/dist"
[ -d "{app_dir}/build" ] && BUILD_DIR="{app_dir}/build"
[ -d "{app_dir}/out" ] && BUILD_DIR="{app_dir}/out"

# Install Caddy
if ! command -v caddy &>/dev/null; then
    apt-get install -y debian-keyring debian-archive-keyring apt-transport-https
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/deb/debian/config/deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
    apt-get update
    apt-get install -y caddy
fi

# Configure Caddy for static files
cat > /etc/caddy/Caddyfile <<CADDYEOF
:80 {{
    root * $BUILD_DIR
    file_server
    encode gzip
}}
{domain_extra}
CADDYEOF

systemctl enable caddy
systemctl restart caddy

echo "Static site deployed"

"#,
        node_version = config.node_version,
        app_dir = app_dir,
        domain_extra = if config.domain.is_empty() {
            String::new()
        } else {
            format!("\n{domain} {{\n    root * $BUILD_DIR\n    file_server\n    encode gzip\n}}", domain = config.domain)
        },
    )
}

fn docker_compose_full(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    format!(r#"# ---------- Docker Compose Full Stack ----------
cd "{app_dir}"
docker compose up -d --build

# Wait for app to be healthy
echo "Waiting for app to start..."
for i in $(seq 1 60); do
    if curl -sf http://localhost:{port} > /dev/null 2>&1; then
        echo "App is up after $((i * 5)) seconds"
        break
    fi
    sleep 5
done

echo "Docker Compose stack started"

"#,
        app_dir = app_dir,
        port = config.app_port,
    )
}

fn caddy_setup(config: &ProvisioningConfig) -> String {
    let caddyfile = if config.domain.is_empty() {
        // No domain -- serve on port 80 only
        format!(
            ":80 {{\n    reverse_proxy localhost:{port}\n    encode gzip\n}}",
            port = config.app_port,
        )
    } else {
        // Domain configured -- serve on both :80 (IP access) and domain (auto-HTTPS when DNS is ready)
        // Use on_demand TLS so Caddy doesn't fail if DNS isn't pointed yet
        format!(
            ":80 {{\n    reverse_proxy localhost:{port}\n    encode gzip\n}}\n\n{domain} {{\n    reverse_proxy localhost:{port}\n    encode gzip\n}}",
            port = config.app_port,
            domain = config.domain,
        )
    };

    format!(r#"# ---------- Caddy reverse proxy ----------
# Install Caddy via direct binary download (most reliable across all distros)
if ! command -v caddy &>/dev/null; then
    CADDY_ARCH=$(dpkg --print-architecture 2>/dev/null || uname -m)
    # Normalize architecture name
    case "$CADDY_ARCH" in
        aarch64) CADDY_ARCH="arm64" ;;
        x86_64)  CADDY_ARCH="amd64" ;;
    esac

    echo "Installing Caddy for $CADDY_ARCH..."
    curl -o /usr/local/bin/caddy -L "https://caddyserver.com/api/download?os=linux&arch=$CADDY_ARCH"
    chmod +x /usr/local/bin/caddy
    echo "Caddy version: $(/usr/local/bin/caddy version)"

    # Create caddy user and group
    groupadd --system caddy 2>/dev/null || true
    useradd --system --gid caddy --create-home --home-dir /var/lib/caddy --shell /usr/sbin/nologin caddy 2>/dev/null || true

    # Create systemd service
    cat > /etc/systemd/system/caddy.service <<'SVCEOF'
[Unit]
Description=Caddy
After=network.target network-online.target
Requires=network-online.target

[Service]
Type=notify
ExecStart=/usr/local/bin/caddy run --environ --config /etc/caddy/Caddyfile
ExecReload=/usr/local/bin/caddy reload --config /etc/caddy/Caddyfile
TimeoutStopSec=5s
LimitNOFILE=1048576
PrivateTmp=true
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
SVCEOF
    systemctl daemon-reload
fi
mkdir -p /etc/caddy

# Configure Caddyfile
cat > /etc/caddy/Caddyfile <<'CADDYEOF'
{caddyfile}
CADDYEOF

systemctl enable caddy
systemctl restart caddy

echo "Caddy configured"

"#,
        caddyfile = caddyfile,
    )
}

fn deploy_helper(config: &ProvisioningConfig) -> String {
    let app_dir = format!("/opt/{}", config.project_name);
    let redeploy = match config.stack_type.as_str() {
        "nextjs-docker-compose" | "nextjs-standalone" => {
            format!("cd {app_dir} && git pull origin {branch} && npm ci --production=false && npm run build && pm2 restart {project}",
                app_dir = app_dir, branch = config.github_branch, project = config.project_name)
        }
        "docker-compose-full" | "node-api-docker" => {
            format!("cd {app_dir} && git pull origin {branch} && docker compose up -d --build",
                app_dir = app_dir, branch = config.github_branch)
        }
        _ => {
            format!("cd {app_dir} && git pull origin {branch} && npm ci && npm run build && pm2 restart {project}",
                app_dir = app_dir, branch = config.github_branch, project = config.project_name)
        }
    };

    format!(r#"# ---------- Deploy helper script ----------
cat > /opt/deploy.sh <<'DEPLOYEOF'
#!/bin/bash
set -euo pipefail
{redeploy}
echo "Deploy completed at $(date)"
DEPLOYEOF
chmod +x /opt/deploy.sh

"#,
        redeploy = redeploy,
    )
}

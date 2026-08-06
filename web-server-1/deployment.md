# Deployment Guide for `web-server-1`

This document details all deployment options for the **`web-server-1`** Rust application built with Axum and Askama templates.

---

## Technical Overview

- **Single Binary Output**: Askama templates (`templates/index.html`) are compiled directly into the Rust binary executable at compile time. You do not need to ship HTML files separately in production.
- **Port & Address Configuration**: `src/main.rs` listens on `0.0.0.0:8080` by default and respects `SERVER_ADDR` or `PORT` environment variables.

---

## Method 1: Docker Container Deployment (Recommended)

### 1. Build the Docker Image
```bash
docker build -t web-server-1 .
```

### 2. Run the Container
```bash
docker run -d -p 8080:8080 --name web-server web-server-1
```
The server will be available at `http://localhost:8080` (or `http://YOUR_SERVER_IP:8080`).

---

## Method 2: cPanel Shared Hosting Deployment

Standard cPanel hosting is designed for PHP, but you can deploy Rust using cross-compilation, background process management, and Apache proxying.

### Step 1: Cross-Compile Binary for Linux (x86_64)
If building on Windows:
```bash
# Add Linux target
rustup target add x86_64-unknown-linux-musl

# Build release executable
cargo build --release --target x86_64-unknown-linux-musl
```
Binary location: `target/x86_64-unknown-linux-musl/release/web-server-1`

### Step 2: Upload to cPanel
1. Open cPanel **File Manager** or use **FTP/SSH**.
2. Create a folder in your home directory (outside `public_html`), e.g., `/home/username/app/`.
3. Upload `web-server-1` into `/home/username/app/`.
4. Change permissions to executable (`0755` or `chmod +x /home/username/app/web-server-1`).

### Step 3: Keep the Server Running (Cron Job)
In cPanel $\rightarrow$ **Cron Jobs**, add a job to run every minute (or `@reboot`):
```bash
pgrep -f "web-server-1" > /dev/null || PORT=8080 /home/username/app/web-server-1 > /home/username/app/app.log 2>&1 &
```

### Step 4: Configure `.htaccess` Reverse Proxy
In your `public_html/.htaccess` file, add:
```apache
RewriteEngine On
RewriteCond %{HTTP_HOST} ^(www\.)?yourdomain\.com$ [NC]
RewriteRule ^(.*)$ http://127.0.0.1:8080/$1 [P,L]
```

---

## Method 3: Standalone Linux Server (VPS / EC2)

### Step 1: Compile Release Binary
On your server or build machine:
```bash
cargo build --release
```
Binary location: `target/release/web-server-1`

### Step 2: Copy to `/usr/local/bin`
```bash
sudo cp target/release/web-server-1 /usr/local/bin/web-server-1
sudo chmod +x /usr/local/bin/web-server-1
```

### Step 3: Systemd Service Configuration
Create `/etc/systemd/system/web-server-1.service`:
```ini
[Unit]
Description=Web Server 1 Rust Application
After=network.target

[Service]
Type=simple
User=www-data
ExecStart=/usr/local/bin/web-server-1
Restart=on-failure
Environment="SERVER_ADDR=0.0.0.0:8080"

[Install]
WantedBy=multi-user.target
```

### Step 4: Enable & Start Service
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now web-server-1
```

---

## Method 4: Nginx Reverse Proxy with Free SSL (HTTPS)

### 1. Nginx Configuration (`/etc/nginx/sites-available/web-server-1`)
```nginx
server {
    server_name yourdomain.com www.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. Enable & Install SSL Certificate
```bash
sudo ln -s /etc/nginx/sites-available/web-server-1 /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
sudo certbot --nginx -d yourdomain.com -d www.yourdomain.com
```

---

## Method 5: Modern Cloud Platforms (Render, Fly.io, Railway)

- **Render.com**: Connect GitHub repo $\rightarrow$ Select **Web Service** $\rightarrow$ Render will automatically use the included `Dockerfile`.
- **Fly.io**: Install `flyctl` $\rightarrow$ Run `fly launch` in the project root $\rightarrow$ Deploy with `fly deploy`.
- **Railway.app**: Connect GitHub repo $\rightarrow$ Railway auto-detects `Dockerfile` or `Cargo.toml` and builds automatically.

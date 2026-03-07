# Pyrite Cloud CLI

The **Pyrite Cloud CLI** helps you build, configure, and deploy applications to **Pyrite Cloud** directly from your terminal.

It provides utilities for generating secure Dockerfiles, managing deployments, and interacting with your Pyrite Cloud environments.

---

## 📦 Installation

### 🍺 Homebrew (macOS / Linux)

Install using the Pyrite Homebrew tap:

```bash
brew install pyritecloud/tap/pyrite
```

Verify installation:

```bash
pyrite --version
```

Upgrade anytime:

```bash
brew upgrade pyrite
```

---

### 🪟 PowerShell (Windows)

Run the following command in PowerShell:

```powershell
irm "https://github.com/pyritecloud/cli/releases/latest/download/pyrite-installer.ps1" | iex
```

Restart your terminal and verify:

```powershell
pyrite --version
```

---

### 🐳 Run with Docker

You can run the CLI without installing it locally:

```bash
docker run --rm -it ghcr.io/pyritecloud/pyrite:latest
```

Example:

```bash
docker run --rm -it -v "$(pwd)":/tmp ghcr.io/pyritecloud/pyrite:latest docker init
```

---

## 🚀 Getting Started

Initialize a Dockerfile for your project:

```bash
pyrite docker init
```

Follow the prompts to generate a production-ready Dockerfile.

---

## 📚 Documentation

Full documentation is available at:

https://www.pyrite.cloud/docs

---

## 📄 License

MIT License

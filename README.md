# Whispery

🎙️ A lightweight desktop app for **push-to-talk dictation** with AI-powered text transformation.  
Hold a key, speak, release, and get polished text in your clipboard.

## ✨ What it does

- **Global push-to-talk** - Hold a configurable key (Caps Lock, Right Alt, etc.) to record
- **Animated overlay orb** - Shows listening/processing/success states
- **Whisper transcription** - Accurate speech-to-text via OpenAI Whisper API
- **Smart prompt transforms** - Fix grammar, summarize, make text professional, generate code, and more
- **Multiple LLM providers** - OpenAI, Anthropic, or OpenAI-compatible endpoints
- **Tray-first app** - Runs quietly in the background
- **Flexible output** - Auto-copy, auto-insert, and insertion mode controls

## 🧱 Tech stack

- **Tauri v2** (Rust backend + webview frontend)
- **Svelte 5** + TypeScript
- **Tailwind CSS v4**
- **cpal** for cross-platform audio capture
- **reqwest** for API communication

## 🚀 Clone and run locally

### 1) Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) (18+)
- npm (comes with Node)

### 2) Clone

```bash
git clone https://github.com/ChristianTrummer99/whispery.git
cd whispery
```

### 3) Install dependencies

```bash
npm install
```

### 4) Run in development mode

```bash
npm run tauri dev
```

### 5) Build production bundles

```bash
npm run tauri build
```

## ⚙️ First-time app setup

1. Launch the app (settings window opens)
2. Open **API Keys** and add your OpenAI key for Whisper
3. Open **Audio** and choose your microphone + push-to-talk key
4. Open **Prompts** and pick/create transformation prompts
5. Hold your push-to-talk key, speak, then release to process
6. In **Audio > Output After Transcription**, choose auto-copy / auto-insert behavior

## 📦 Releases and updater bundles

Normal users can download releases from this repo.  
Updater artifacts are published to **`ChristianTrummer99/whispery-updates`** so the app can fetch updates without requiring auth.

- Public updater repo: `https://github.com/ChristianTrummer99/whispery-updates`
- It contains release bundles like `latest.json` and signed update assets used by in-app updates
- Release workflow: `.github/workflows/release.yml`

## 🔐 Maintainer notes (signing + release pipeline)

If you maintain releases, set up updater signing once:

```bash
npm run tauri signer generate -- -w ~/.tauri/whispery.key
```

Then configure:

- `src-tauri/tauri.conf.json` -> `plugins.updater.pubkey` (public key)
- GitHub Actions secrets:
  - `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - `UPDATES_REPO_TOKEN` (write access to `ChristianTrummer99/whispery-updates`)

For signed macOS releases, also set:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD` (app-specific password)
- `APPLE_TEAM_ID`

Publish a new version by tagging:

```bash
git tag v0.1.1
git push origin v0.1.1
```

## 🗂️ Project structure

```text
whispery/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── stores.ts             # Settings persistence
│   │   └── components/           # UI components
│   └── routes/
│       ├── +page.svelte          # Main settings dashboard
│       └── overlay/+page.svelte  # Floating overlay widget
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── lib.rs                # Tauri commands & app setup
│       ├── audio.rs              # Audio capture via cpal
│       ├── transcribe.rs         # Whisper API integration
│       └── transform.rs          # LLM text transformation
└── package.json
```

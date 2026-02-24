# Whispery

A lightweight desktop app for **push-to-talk dictation** with AI-powered text transformation. Hold a key, speak, and get polished text copied to your clipboard.

## Features

- **Global Push-to-Talk** — Hold a configurable key (Caps Lock, Right Alt, etc.) to record, release to process
- **Animated Overlay** — A small always-on-top orb shows listening/processing/success state with smooth animations
- **Speech-to-Text** — Powered by OpenAI Whisper API for high-accuracy transcription
- **Transformation Prompts** — Apply AI prompts to your speech: fix grammar, summarize, make professional, turn into code, or create your own
- **Multiple LLM Providers** — OpenAI, Anthropic, or any custom OpenAI-compatible endpoint
- **System Tray** — Runs quietly in the background, accessible from the tray icon
- **Clipboard Output** — Transformed text is automatically copied to your clipboard

## Tech Stack

- **Tauri v2** (Rust backend + webview frontend)
- **Svelte 5** + TypeScript
- **Tailwind CSS v4**
- **cpal** for cross-platform audio capture
- **reqwest** for API communication

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- An OpenAI API key (for Whisper transcription)

## Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Configuration

1. Launch the app — the settings window opens
2. Go to **API Keys** tab and enter your OpenAI API key
3. Go to **Audio** tab to select your microphone and PTT key
4. Go to **Prompts** tab to choose or create transformation prompts
5. Hold your PTT key to dictate — the overlay orb appears
6. Release the key — text is transcribed, transformed, and copied to clipboard

## Project Structure

```
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

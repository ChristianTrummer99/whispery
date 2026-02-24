import { load, Store } from "@tauri-apps/plugin-store";

export interface Prompt {
  id: string;
  name: string;
  template: string;
}

export interface Settings {
  openaiApiKey: string;
  llmProvider: "openai" | "anthropic" | "custom";
  llmApiKey: string;
  llmApiUrl: string;
  llmModel: string;
  pttKey: string;
  pttMode: "hold" | "toggle";
  whisperLanguage: string;
  selectedDevice: string;
  activePromptId: string;
  prompts: Prompt[];
}

export const WHISPER_LANGUAGES = [
  { code: "en", name: "English" },
  { code: "zh", name: "Chinese" },
  { code: "de", name: "German" },
  { code: "es", name: "Spanish" },
  { code: "ru", name: "Russian" },
  { code: "ko", name: "Korean" },
  { code: "fr", name: "French" },
  { code: "ja", name: "Japanese" },
  { code: "pt", name: "Portuguese" },
  { code: "tr", name: "Turkish" },
  { code: "pl", name: "Polish" },
  { code: "ca", name: "Catalan" },
  { code: "nl", name: "Dutch" },
  { code: "ar", name: "Arabic" },
  { code: "sv", name: "Swedish" },
  { code: "it", name: "Italian" },
  { code: "id", name: "Indonesian" },
  { code: "hi", name: "Hindi" },
  { code: "fi", name: "Finnish" },
  { code: "vi", name: "Vietnamese" },
  { code: "he", name: "Hebrew" },
  { code: "uk", name: "Ukrainian" },
  { code: "el", name: "Greek" },
  { code: "ms", name: "Malay" },
  { code: "cs", name: "Czech" },
  { code: "ro", name: "Romanian" },
  { code: "da", name: "Danish" },
  { code: "hu", name: "Hungarian" },
  { code: "ta", name: "Tamil" },
  { code: "no", name: "Norwegian" },
  { code: "th", name: "Thai" },
  { code: "ur", name: "Urdu" },
  { code: "hr", name: "Croatian" },
  { code: "bg", name: "Bulgarian" },
  { code: "lt", name: "Lithuanian" },
  { code: "la", name: "Latin" },
  { code: "sk", name: "Slovak" },
  { code: "sl", name: "Slovenian" },
  { code: "et", name: "Estonian" },
  { code: "lv", name: "Latvian" },
] as const;

export const DEFAULT_PROMPTS: Prompt[] = [
  {
    id: "raw",
    name: "Raw Transcription",
    template: "",
  },
  {
    id: "fix-grammar",
    name: "Fix Grammar",
    template:
      "Fix the grammar and punctuation of the following text, preserving the original meaning:\n\n{text}",
  },
  {
    id: "professional",
    name: "Make Professional",
    template:
      "Rewrite the following text to sound more professional and polished:\n\n{text}",
  },
  {
    id: "summarize",
    name: "Summarize",
    template: "Summarize the following text concisely:\n\n{text}",
  },
  {
    id: "code",
    name: "Turn into Code",
    template:
      "Convert the following natural language description into clean, well-structured code. Infer the best language from context:\n\n{text}",
  },
];

const LLM_URLS: Record<string, string> = {
  openai: "https://api.openai.com/v1/chat/completions",
  anthropic: "https://api.anthropic.com/v1/messages",
};

const LLM_MODELS: Record<string, string> = {
  openai: "gpt-4o-mini",
  anthropic: "claude-sonnet-4-20250514",
};

export function getLlmUrl(settings: Settings): string {
  if (settings.llmProvider === "custom") return settings.llmApiUrl;
  return LLM_URLS[settings.llmProvider] ?? settings.llmApiUrl;
}

export function getLlmModel(settings: Settings): string {
  if (settings.llmProvider === "custom") return settings.llmModel;
  return settings.llmModel || LLM_MODELS[settings.llmProvider] || "gpt-4o-mini";
}

export function getLlmApiKey(settings: Settings): string {
  return settings.llmApiKey || settings.openaiApiKey;
}

export const DEFAULT_SETTINGS: Settings = {
  openaiApiKey: "",
  llmProvider: "openai",
  llmApiKey: "",
  llmApiUrl: "https://api.openai.com/v1/chat/completions",
  llmModel: "gpt-4o-mini",
  pttKey: "CapsLock",
  pttMode: "hold",
  whisperLanguage: "en",
  selectedDevice: "",
  activePromptId: "fix-grammar",
  prompts: [...DEFAULT_PROMPTS],
};

let store: Store | null = null;

export async function getStore(): Promise<Store> {
  if (!store) {
    store = await load("settings.json", { autoSave: true });
  }
  return store;
}

export async function loadSettings(): Promise<Settings> {
  const s = await getStore();
  const saved = await s.get<Settings>("settings");
  if (saved) {
    return { ...DEFAULT_SETTINGS, ...saved };
  }
  return { ...DEFAULT_SETTINGS };
}

export async function saveSettings(settings: Settings): Promise<void> {
  const s = await getStore();
  await s.set("settings", settings);
  await s.save();
}

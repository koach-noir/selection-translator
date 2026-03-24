import { invoke } from "@tauri-apps/api/core";
import type { Config, TranslationEntry } from "./types";

export async function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

export async function updateConfig(config: Config): Promise<void> {
  return invoke("update_config", { config });
}

export async function getEnabled(): Promise<boolean> {
  return invoke<boolean>("get_enabled");
}

export async function setEnabled(enabled: boolean): Promise<void> {
  return invoke("set_enabled", { enabled });
}

export async function translateText(
  text: string
): Promise<string | null> {
  return invoke<string | null>("translate_text", { text });
}

export async function getPopupEntries(): Promise<TranslationEntry[]> {
  return invoke<TranslationEntry[]>("get_popup_entries");
}

export async function dismissPopupEntry(id: number): Promise<number> {
  return invoke<number>("dismiss_popup_entry", { id });
}

export async function clearPopupSession(): Promise<void> {
  return invoke("clear_popup_session");
}

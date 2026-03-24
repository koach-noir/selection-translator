import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import {
  getPopupEntries,
  dismissPopupEntry,
  clearPopupSession,
} from "../shared/tauri-bridge";
import type { TranslationEntry } from "../shared/types";

let currentEntries: readonly TranslationEntry[] = [];

const ENTRY_MAX_HEIGHT_RATIO = 0.2;

function getEntryMaxHeight(): number {
  return Math.round(window.screen.height * ENTRY_MAX_HEIGHT_RATIO);
}

function renderEntries(entries: readonly TranslationEntry[]): void {
  const container = document.getElementById("entries-container");
  if (!container) return;

  container.innerHTML = "";
  const maxHeight = getEntryMaxHeight();

  entries.forEach((entry, index) => {
    const isCurrent = index === entries.length - 1;
    const entryEl = createEntryElement(entry, isCurrent, maxHeight);
    container.appendChild(entryEl);
  });
}

function createEntryElement(
  entry: TranslationEntry,
  isCurrent: boolean,
  maxHeight: number,
): HTMLElement {
  const entryEl = document.createElement("div");
  entryEl.className = isCurrent ? "entry entry-current" : "entry";
  entryEl.dataset.id = String(entry.id);

  // ヘッダー: 閉じるボタン
  const headerEl = document.createElement("div");
  headerEl.className = "entry-header";
  const closeBtn = document.createElement("button");
  closeBtn.className = "entry-close";
  closeBtn.textContent = "×";
  closeBtn.addEventListener("click", () => handleDismiss(entry.id));
  headerEl.appendChild(closeBtn);

  // コンテンツ: 翻訳文(上) → 原文(下)
  const contentEl = document.createElement("div");
  contentEl.className = "entry-content";
  contentEl.style.maxHeight = `${maxHeight}px`;

  const translatedEl = document.createElement("div");
  translatedEl.className = "entry-translated";
  translatedEl.textContent = entry.translated;

  const originalEl = document.createElement("div");
  originalEl.className = "entry-original";
  originalEl.textContent = entry.original;

  contentEl.appendChild(translatedEl);
  contentEl.appendChild(originalEl);

  // 展開/畳むトグル（オーバーフロー時のみ表示）
  const toggleBtn = document.createElement("button");
  toggleBtn.className = "entry-toggle";
  toggleBtn.textContent = "▼";
  toggleBtn.style.display = "none";
  toggleBtn.addEventListener("click", () => {
    const isCollapsed = contentEl.style.maxHeight !== "none";
    contentEl.style.maxHeight = isCollapsed ? "none" : `${maxHeight}px`;
    toggleBtn.textContent = isCollapsed ? "▲" : "▼";
    adjustWindowLayout();
  });

  entryEl.appendChild(headerEl);
  entryEl.appendChild(contentEl);
  entryEl.appendChild(toggleBtn);

  // オーバーフロー検出は次フレームで実行
  requestAnimationFrame(() => {
    if (contentEl.scrollHeight > maxHeight) {
      toggleBtn.style.display = "block";
    }
  });

  return entryEl;
}

async function handleDismiss(id: number): Promise<void> {
  const remaining = await dismissPopupEntry(id);
  currentEntries = currentEntries.filter((e) => e.id !== id);
  if (remaining === 0) {
    await getCurrentWindow().hide();
    return;
  }
  renderEntries(currentEntries);
  await adjustWindowLayout();
}

function handleNewTranslation(entry: TranslationEntry): void {
  const updated = [...currentEntries, entry].slice(-5);
  currentEntries = updated;
  renderEntries(updated);
  adjustWindowLayout();
}

async function adjustWindowLayout(): Promise<void> {
  const appWindow = getCurrentWindow();
  const screenWidth = window.screen.width;
  const screenHeight = window.screen.height;
  const windowWidth = Math.round(screenWidth / 4);

  // 次フレームでレイアウト確定後に計測
  await new Promise((r) => requestAnimationFrame(r));

  const container = document.getElementById("entries-container");
  const contentHeight = container
    ? Math.max(container.scrollHeight + 16, 60)
    : 120;

  await appWindow.setSize(new LogicalSize(windowWidth, contentHeight));

  const lastEntry = container?.lastElementChild as HTMLElement | null;
  const lastEntryCenter = lastEntry
    ? lastEntry.offsetTop + lastEntry.offsetHeight / 2
    : contentHeight / 2;

  const x = screenWidth - windowWidth;
  const y = Math.max(0, Math.round(screenHeight / 2 - lastEntryCenter));

  await appWindow.setPosition(new LogicalPosition(x, y));
}

function setupKeyboardClose(): void {
  document.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      currentEntries = [];
      await clearPopupSession();
      await getCurrentWindow().hide();
    }
  });
}

async function init(): Promise<void> {
  const entries = await getPopupEntries();
  currentEntries = entries;
  renderEntries(entries);

  await listen<TranslationEntry>("translation-new", (event) => {
    handleNewTranslation(event.payload);
  });

  setupKeyboardClose();
  await adjustWindowLayout();
}

document.addEventListener("DOMContentLoaded", init);

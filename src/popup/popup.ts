import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import {
  getConfig,
  getPopupEntries,
  dismissPopupEntry,
  clearPopupSession,
} from "../shared/tauri-bridge";
import type { TranslationEntry, CopyMode } from "../shared/types";

// ─── Icons (Lucide) ───

const ICON_COPY = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>`;

const ICON_CHECK = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`;

const ICON_CLOSE = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>`;

// ─── State ───

let currentEntries: readonly TranslationEntry[] = [];
let hoveredId: number | null = null;
let copyMode: CopyMode = "translated";

const ENTRY_MAX_HEIGHT_RATIO = 0.2;
const COPY_FEEDBACK_MS = 1200;

// ─── Highlight ───

function getHighlightedId(): number | null {
  if (hoveredId !== null) return hoveredId;
  const last = currentEntries[currentEntries.length - 1];
  return last?.id ?? null;
}

function updateHighlights(): void {
  const id = getHighlightedId();
  document.querySelectorAll<HTMLElement>(".entry-card").forEach((el) => {
    el.classList.toggle("highlighted", Number(el.dataset.id) === id);
  });
}

// ─── Copy ───

function formatEntry(entry: TranslationEntry, mode: CopyMode): string {
  switch (mode) {
    case "original":
      return entry.original;
    case "both":
      return `${entry.original} → ${entry.translated}`;
    default:
      return entry.translated;
  }
}

async function flashCopied(btn: HTMLElement): Promise<void> {
  const prev = btn.innerHTML;
  btn.innerHTML = ICON_CHECK;
  btn.classList.add("copied");
  await new Promise((r) => setTimeout(r, COPY_FEEDBACK_MS));
  btn.innerHTML = prev;
  btn.classList.remove("copied");
}

async function copyEntry(entry: TranslationEntry, btn: HTMLElement): Promise<void> {
  await navigator.clipboard.writeText(formatEntry(entry, copyMode));
  flashCopied(btn);
}

async function copyAll(btn: HTMLElement): Promise<void> {
  const text = currentEntries.map((e) => formatEntry(e, copyMode)).join("\n");
  await navigator.clipboard.writeText(text);
  flashCopied(btn);
}

// ─── Render ───

function getEntryMaxHeight(): number {
  return Math.round(window.screen.height * ENTRY_MAX_HEIGHT_RATIO);
}

function renderAll(): void {
  renderEntries(currentEntries);
  renderFooter(currentEntries.length > 0);
  updateHighlights();
}

function renderEntries(entries: readonly TranslationEntry[]): void {
  const container = document.getElementById("entries-container");
  if (!container) return;

  container.innerHTML = "";
  const maxHeight = getEntryMaxHeight();

  entries.forEach((entry) => {
    container.appendChild(createCard(entry, maxHeight));
  });

  // ホバーでハイライトを排他的に移動
  container.addEventListener("mouseleave", () => {
    hoveredId = null;
    updateHighlights();
  });
}

function createCard(entry: TranslationEntry, maxHeight: number): HTMLElement {
  const card = document.createElement("div");
  card.className = "entry-card";
  card.dataset.id = String(entry.id);

  card.addEventListener("mouseenter", () => {
    hoveredId = entry.id;
    updateHighlights();
  });

  // ヘッダー: コピー・閉じるボタン
  const header = document.createElement("div");
  header.className = "card-header";

  const copyBtn = createActionBtn(ICON_COPY, "Copy");
  copyBtn.addEventListener("click", () => copyEntry(entry, copyBtn));

  const closeBtn = createActionBtn(ICON_CLOSE, "Close");
  closeBtn.addEventListener("click", () => handleDismiss(entry.id));

  header.appendChild(copyBtn);
  header.appendChild(closeBtn);

  // コンテンツ: 翻訳文 → 原文
  const content = document.createElement("div");
  content.className = "card-content";
  content.style.maxHeight = `${maxHeight}px`;

  const translatedEl = document.createElement("div");
  translatedEl.className = "card-translated";
  translatedEl.textContent = entry.translated;

  const originalEl = document.createElement("div");
  originalEl.className = "card-original";
  originalEl.textContent = entry.original;

  content.appendChild(translatedEl);
  content.appendChild(originalEl);

  // オーバーフロー時のトグル
  const toggleBtn = document.createElement("button");
  toggleBtn.className = "card-toggle";
  toggleBtn.textContent = "▼";
  toggleBtn.style.display = "none";
  toggleBtn.addEventListener("click", () => {
    const collapsed = content.style.maxHeight !== "none";
    content.style.maxHeight = collapsed ? "none" : `${maxHeight}px`;
    toggleBtn.textContent = collapsed ? "▲" : "▼";
    adjustWindowLayout();
  });

  card.appendChild(header);
  card.appendChild(content);
  card.appendChild(toggleBtn);

  // オーバーフロー検出
  requestAnimationFrame(() => {
    if (content.scrollHeight > maxHeight) {
      toggleBtn.style.display = "block";
    }
  });

  return card;
}

function createActionBtn(icon: string, title: string): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.className = "action-btn";
  btn.innerHTML = icon;
  btn.title = title;
  return btn;
}

function renderFooter(visible: boolean): void {
  let footer = document.getElementById("popup-footer");

  if (!visible) {
    footer?.remove();
    return;
  }

  if (!footer) {
    footer = document.createElement("div");
    footer.id = "popup-footer";
    document.getElementById("popup-root")?.appendChild(footer);
  }

  footer.innerHTML = "";
  const btn = document.createElement("button");
  btn.className = "copy-all-btn";
  btn.innerHTML = `${ICON_COPY} Copy All`;
  btn.addEventListener("click", () => copyAll(btn));
  footer.appendChild(btn);
}

// ─── Actions ───

async function handleDismiss(id: number): Promise<void> {
  const remaining = await dismissPopupEntry(id);
  currentEntries = currentEntries.filter((e) => e.id !== id);

  if (remaining === 0) {
    await getCurrentWindow().hide();
    return;
  }

  hoveredId = null;
  renderAll();
  await adjustWindowLayout();
}

function handleNewTranslation(entry: TranslationEntry): void {
  currentEntries = [...currentEntries, entry].slice(-5);
  hoveredId = null;
  renderAll();
  adjustWindowLayout();
}

// ─── Layout ───

async function adjustWindowLayout(): Promise<void> {
  const appWindow = getCurrentWindow();
  const screenWidth = window.screen.width;
  const screenHeight = window.screen.height;
  const windowWidth = Math.round(screenWidth / 4);

  await new Promise((r) => requestAnimationFrame(r));

  const root = document.getElementById("popup-root");
  const contentHeight = root
    ? Math.max(root.scrollHeight + 4, 60)
    : 120;

  await appWindow.setSize(new LogicalSize(windowWidth, contentHeight));

  const container = document.getElementById("entries-container");
  const lastCard = container?.lastElementChild as HTMLElement | null;
  const lastCardCenter = lastCard
    ? lastCard.offsetTop + lastCard.offsetHeight / 2
    : contentHeight / 2;

  const x = screenWidth - windowWidth;
  const y = Math.max(0, Math.round(screenHeight / 2 - lastCardCenter));

  await appWindow.setPosition(new LogicalPosition(x, y));
}

// ─── Keyboard ───

function setupKeyboardClose(): void {
  document.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      currentEntries = [];
      await clearPopupSession();
      await getCurrentWindow().hide();
    }
  });
}

// ─── Init ───

async function init(): Promise<void> {
  const config = await getConfig();
  copyMode = config.copyMode;

  const entries = await getPopupEntries();
  currentEntries = entries;
  renderAll();

  await listen<TranslationEntry>("translation-new", (event) => {
    handleNewTranslation(event.payload);
  });

  setupKeyboardClose();
  await adjustWindowLayout();
}

document.addEventListener("DOMContentLoaded", init);

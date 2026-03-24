import { getConfig, updateConfig } from "../shared/tauri-bridge";
import type { Config } from "../shared/types";

function getInput<T extends HTMLInputElement>(id: string): T {
  return document.getElementById(id) as T;
}

function populateForm(config: Config): void {
  const opacityInput = getInput("opacity");
  opacityInput.value = String(config.opacity);
  const opacityValue = document.getElementById("opacity-value");
  if (opacityValue) opacityValue.textContent = String(config.opacity);

  getInput("fontSize").value = String(config.fontSize);
  getInput("fontColor").value = config.fontColor;
  getInput("backgroundColor").value = config.backgroundColor;
  getInput("borderRadius").value = String(config.borderRadius);
  getInput("shadow").checked = config.shadow;
  getInput("minLength").value = String(config.minLength);
  getInput("englishOnly").checked = config.englishOnly;
}

// 現在のconfigを保持し、フォームにない項目を保全する
let currentConfig: Config | null = null;

function readForm(): Config {
  return {
    ...(currentConfig ?? ({} as Config)),
    enabled: true,
    opacity: parseFloat(getInput("opacity").value),
    fontSize: parseInt(getInput("fontSize").value, 10),
    fontColor: getInput("fontColor").value,
    backgroundColor: getInput("backgroundColor").value,
    borderRadius: parseInt(getInput("borderRadius").value, 10),
    shadow: getInput("shadow").checked,
    minLength: parseInt(getInput("minLength").value, 10),
    englishOnly: getInput("englishOnly").checked,
  };
}

function showStatus(message: string): void {
  const status = document.getElementById("status");
  if (status) {
    status.textContent = message;
    setTimeout(() => {
      status.textContent = "";
    }, 2000);
  }
}

async function init(): Promise<void> {
  const config = await getConfig();
  currentConfig = config;
  populateForm(config);

  const opacityInput = getInput("opacity");
  opacityInput.addEventListener("input", () => {
    const opacityValue = document.getElementById("opacity-value");
    if (opacityValue) opacityValue.textContent = opacityInput.value;
  });

  const saveBtn = document.getElementById("save-btn");
  if (saveBtn) {
    saveBtn.addEventListener("click", async () => {
      const newConfig = readForm();
      try {
        await updateConfig(newConfig);
        showStatus("Saved");
      } catch (e) {
        showStatus(`Error: ${e}`);
      }
    });
  }
}

document.addEventListener("DOMContentLoaded", init);

export type CopyMode = "translated" | "original" | "both";

export type PopupPosition =
  | "rightQuarter"
  | "rightHalf"
  | "leftQuarter"
  | "leftHalf";

export interface Config {
  enabled: boolean;
  opacity: number;
  fontSize: number;
  fontColor: string;
  backgroundColor: string;
  borderRadius: number;
  shadow: boolean;
  minLength: number;
  englishOnly: boolean;
  copyMode: CopyMode;
  popupPosition: PopupPosition;
}

export const DEFAULT_CONFIG: Config = {
  enabled: true,
  opacity: 0.9,
  fontSize: 14,
  fontColor: "#ffffff",
  backgroundColor: "#222222",
  borderRadius: 8,
  shadow: true,
  minLength: 4,
  englishOnly: true,
  copyMode: "translated",
  popupPosition: "rightQuarter",
};

export interface TranslationEntry {
  readonly id: number;
  readonly original: string;
  readonly translated: string;
  readonly timestamp: number;
}

import { create } from "zustand";

export type ThemeMode = "light" | "dark";

interface ThemeState {
  mode: ThemeMode;
  toggle: () => void;
  setMode: (mode: ThemeMode) => void;
}

const getInitialMode = (): ThemeMode => {
  if (typeof window === "undefined") {
    return "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

export const useThemeStore = create<ThemeState>((set) => ({
  mode: getInitialMode(),
  toggle: () =>
    set((state) => ({
      mode: state.mode === "dark" ? "light" : "dark",
    })),
  setMode: (mode) => set({ mode }),
}));

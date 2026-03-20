import { Toaster as SonnerToaster } from "sonner";
import { useThemeStore } from "../../lib/theme-store";

export function Toaster() {
  const mode = useThemeStore((state) => state.mode);

  return (
    <SonnerToaster
      theme={mode}
      richColors
      position="top-right"
      toastOptions={{
        className: "font-sans",
      }}
    />
  );
}

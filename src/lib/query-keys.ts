export const queryKeys = {
  systemInfo: ["systemInfo"] as const,
  allPackages: ["allPackages"] as const,
  orphans: ["orphans"] as const,
  removalHistory: ["removalHistory"] as const,
  removalPreview: (name: string, source: string) =>
    ["removalPreview", name, source] as const,
};

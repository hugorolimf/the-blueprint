declare module "claude-code" {
  interface PluginState {
    "blueprint-terminal": { visible: boolean; image: boolean; pasta: string; estado: { texto: string; linha: string } };
  }
}

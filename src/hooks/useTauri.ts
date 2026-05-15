import { invoke } from "@tauri-apps/api/core";

export interface VaultEntry {
  id: number;
  website: string;
  username: string;
  secret_key: string;
  pashword: string;
  created_at: string;
  updated_at: string;
}

export const commands = {
  generatePassword: (website: string, username: string, secretKey: string, length: number) =>
    invoke<string>("generate_password", { website, username, secretKey, length }),

  setupVault: (masterPassword: string) =>
    invoke<void>("setup_vault", { masterPassword }),

  unlockVault: (masterPassword: string) =>
    invoke<boolean>("unlock_vault", { masterPassword }),

  isVaultInitialized: () =>
    invoke<boolean>("is_vault_initialized"),

  saveEntry: (website: string, username: string, secretKey: string, pashword: string) =>
    invoke<number>("save_entry", { website, username, secretKey, pashword }),

  listEntries: () =>
    invoke<VaultEntry[]>("list_entries"),

  deleteEntry: (id: number) =>
    invoke<void>("delete_entry", { id }),
};

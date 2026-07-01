import { invoke } from "@tauri-apps/api/core";
import { Masternode } from "./masternode";

export class VPS {
  private isInitialized = false;
  constructor(
    public name: string,
    public authentication: "password" | "key",
    public username: string,
    // Either the private key path or password based on the
    // authentication type
    public secret: string,
    public ipAddress: string,
    // @ts-ignore
    private masternodes: Masternode[],
  ) {}

  // Run the first time to setup the docker
  async setupVps() {
    if (this.isInitialized) throw new Error("Calling setupVps twice");
    switch (this.authentication) {
      case "password":
        await invoke("init_vps_with_password", {
          ip_address: this.ipAddress,
          username: this.username,
          password: this.secret,
        });
        break;
      case "key":
        await invoke("init_vps_with_password", {
          ip_address: this.ipAddress,
          username: this.username,
          private_key_path: this.secret,
        });
        break;
    }
    this.isInitialized = true;
  }
}

import { invoke } from '@tauri-apps/api/core';
import { Window } from "@tauri-apps/api/window";

document.getElementById("add-command-form")?.addEventListener("submit", async (event) => {
  event.preventDefault();

  let titleElement = document.getElementById("title") as HTMLInputElement;
  let commandElement = document.getElementById("command") as HTMLInputElement;

  invoke(
    "write_commands", 
    {
      "title": titleElement?.value,
      "command": commandElement?.value,
    },
  );

  Window.getCurrent().close();

  invoke("read_commands");
});

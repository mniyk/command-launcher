import { invoke } from '@tauri-apps/api/core';
import { Window } from "@tauri-apps/api/window";
import { listen, emit } from "@tauri-apps/api/event";

// ウィンドウのロード完了を通知
// ウィンドウのロードを完了を待って、#commandの書き換えを行うため
window.addEventListener("DOMContentLoaded", () => {
  emit(
    "confirmation-window-loaded", 
    "confirmation"
  );
});

// #commandを更新
listen("update-command", (event) => {
  const command = event.payload as string;
    
  const commandElement = document.getElementById("command");

  if (commandElement) {
    commandElement.innerText = command;
  };
});

// ウィンドウを閉じる
window.addEventListener("DOMContentLoaded", () => {
  const noElement = document.getElementById("no");

  noElement?.addEventListener("click", () => {
    const currentWindow = Window.getCurrent();

    currentWindow.close();
  })
});

// コマンドの実行
window.addEventListener("DOMContentLoaded", () => {
  const yesElement = document.getElementById("yes");

  yesElement?.addEventListener("click", async () => {
    const commandElement = document.getElementById("command");

    await invoke(
      "run_command", 
      { 
        args: commandElement?.innerText,
      }
    );

    const currentWindow = Window.getCurrent();

    currentWindow.close();
  })
});

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let lastHeight = 0;

// liタグの作成
listen("commands", (event) => {
  const commandsElement = document.getElementById("commands");
  if (commandsElement) {
    commandsElement.innerHTML = "";
  }

  const commands = event.payload;

  if (Array.isArray(commands)) {
    commands.forEach((item) => {
      let liElement = document.createElement("li");
      liElement.textContent = item.title;
      liElement.setAttribute("data-value", item.command);

      liElement.addEventListener("click", async function (event) {
        const clickedElement = event.target as HTMLInputElement;

        invoke(
          "open_confirmation_window",
          {
            "command": clickedElement.dataset.value,
          }
        );
      }
    )

      commandsElement?.appendChild(liElement);
    })
  }

  move_window_to_bottom_right();
});

// commands.jsonの読込
window.addEventListener("DOMContentLoaded", () => {
  invoke("read_commands");
})

// ウィンドウサイズを変更し、右下に移動
export function move_window_to_bottom_right() {
  const contentHeight = document.body.scrollHeight + 25;
 
  if (Math.abs(contentHeight - lastHeight) > 1) {
    lastHeight = contentHeight;

    invoke(
      "resize_window", 
      { 
        width: 200,
        height: contentHeight,
      }
    );

    invoke("move_window_to_bottom_right");
  }
}

// add_commandウィンドウを開く
listen("open_add_command", async () => {
    await invoke("open_add_command_window");
});

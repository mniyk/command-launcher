use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::Write;

use tauri::{
    Window, 
    LogicalSize, 
    PhysicalPosition,
    WebviewWindowBuilder,
    WebviewUrl,
    Emitter,
    Listener,
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_global_shortcut::{
    Code, 
    Modifiers, 
    ShortcutState
};
use encoding_rs::SHIFT_JIS;
use serde::{ Serialize, Deserialize };
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Command {
    title: String,
    command: String,
}

// commands.jsonの読込
#[tauri::command]
async fn read_commands(window: Window) -> tauri::Result<()> {
    // ファイルが存在するか確認
    if !Path::new("commands.json").exists() {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("commands.json")?;

        file.write_all(b"[]")?;
    }

    let file_content = fs::read_to_string("commands.json")?;
    
    let commands: Vec<Command> = serde_json::from_str(&file_content)?;

    window.emit("commands", commands).unwrap();

    Ok(())
}

// ウィンドウサイズを変更
#[tauri::command]
async fn resize_window(window: Window, width: f64, height: f64) {
    window.set_size(tauri::Size::Logical(LogicalSize {
        width, height,
    }))
    .unwrap();
}

// ウィンドウを右下に移動
#[tauri::command]
async fn move_window_to_bottom_right(window: Window) {
    // モニター情報を取得
    let screen = window.primary_monitor().unwrap();
    let screen_width = screen.as_ref().map_or(
        0, |s| s.size().width as i32
    );
    let screen_height = screen.as_ref().map_or(
        0, |s| s.size().height as i32
    );

    // ウィンドウサイズを取得
    let window_size = window.outer_size().unwrap();
    let window_width = window_size.width as i32;
    let window_height: i32 = window_size.height as i32;

    // 右下の位置を計算
    let x_position = screen_width - window_width + 7;
    let y_position = screen_height - window_height - 41;

    // ウィンドウ位置を設定
    window.set_position(
        PhysicalPosition { x: x_position, y: y_position }
    )
    .unwrap();
}

// confirmationウィンドウを開く
#[tauri::command]
async fn open_confirmation_window(
    app: tauri::AppHandle,
    command: String,
) -> tauri::Result<()> {
    let new_window = WebviewWindowBuilder::new(
        &app,
        "confirmation",
        WebviewUrl::App("confirmation.html".into()),
    )
    .inner_size(600.0, 160.0)
    .center()
    .build()?;

    // ウィンドウのロード完了を受信
    // ウィンドウのロードを完了を待って、#commandの書き換えを行うため
    // フラグを Box でラップして可変性を持たせる
    let has_processed = std::sync::Arc::new(std::sync::Mutex::new(false));
    let has_processed_clone = has_processed.clone();

    app.listen("confirmation-window-loaded", move |event| {
        let mut flag = has_processed_clone.lock().unwrap();
        if *flag {
            return;
        }
        let payload = event.payload();
        if payload == "\"confirmation\"" {
            new_window.emit("update-command", &command).unwrap();
            *flag = true;
        }
    });
    
    Ok(())
}

// コマンドの実行
#[tauri::command]
async fn run_command(app: tauri::AppHandle, args: String) -> tauri::Result<()> {
    let shell = app.shell();

    let output = shell
        .command("powershell.exe")
        .args(vec!["-Command", &args])
        .output()
        .await
        .unwrap();

    let message; 

    // 各出力をShift-JISからUTF-8に変換
    if output.status.success() {
        let (decoded_stdout, _, _) = SHIFT_JIS.decode(&output.stdout);

        message = format!("Success: {}", decoded_stdout);
    } else {
        let (decoded_stderr, _, _) = SHIFT_JIS.decode(&output.stderr);

        message = format!("Failure: {}", decoded_stderr);
    }

    app.notification()
        .builder()
        .title("Command-Launcher Notification")
        .body(message)
        .show()
        .expect("Failed to send notification");

    Ok(())
}

// add_commandウィンドウを開く
#[tauri::command]
async fn open_add_command_window(app: tauri::AppHandle) -> tauri::Result<()> {
    // println!("{}", "test");
    WebviewWindowBuilder::new(
        &app,
        "add-command",
        WebviewUrl::App("add_command.html".into()),
    )
    .inner_size(700.0, 250.0)
    .center()
    .build()?;

    Ok(())
}

#[tauri::command]
async fn write_commands(title: String, command: String) -> tauri::Result<()> {
    let file_content = fs::read_to_string("commands.json")?;
    let mut commands: Value = serde_json::from_str(&file_content)?;

    let new_entry = json!({
        "title": title,
        "command": command,
    });

    if let Some(array) = commands.as_array_mut() {
        array.push(new_entry);
    } 

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("commands.json")?;
    file.write_all(commands.to_string().as_bytes())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            read_commands,
            resize_window,
            move_window_to_bottom_right,
            open_confirmation_window,
            run_command,
            open_add_command_window,
            write_commands,
        ])
        .setup(|app| {
            // ショートカットキーの登録
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["F2"])?
                        .with_handler(|app, shortcut, event| {
                            if event.state == ShortcutState::Pressed  {
                                if shortcut.matches(Modifiers::empty(), Code::F2) {
                                    app.emit(
                                        "open_add_command", 
                                        {}
                                    )
                                    .expect("Failed open_add_command");
                                }
                            }
                        })
                        .build(),
                )?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

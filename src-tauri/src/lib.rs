use tauri_plugin_sql::{Migration, MigrationKind};

mod crypto;
mod sola;
mod printing;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create initial tables",
            sql: include_str!("../migrations/001_initial.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add store logo and theme settings",
            sql: include_str!("../migrations/002_settings_logo_theme.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add sola gateway settings",
            sql: include_str!("../migrations/003_sola_gateway.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "add sola gateway default device id",
            sql: include_str!("../migrations/004_sola_device_id.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add color theme setting",
            sql: include_str!("../migrations/005_color_theme.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add card payment details to payments table",
            sql: include_str!("../migrations/006_payment_card_details.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "add printer settings",
            sql: include_str!("../migrations/007_printer_settings.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:lite-pos.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            crypto::encrypt_value,
            crypto::decrypt_value,
            sola::process_sola_transaction,
            printing::get_system_printers
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

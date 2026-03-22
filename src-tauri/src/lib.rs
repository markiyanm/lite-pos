use tauri_plugin_sql::{Migration, MigrationKind};

mod crypto;
mod logging;
mod sola;
mod printing;
mod recurring_api;
mod customer_sync;

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
        Migration {
            version: 8,
            description: "add printer prompt before print setting",
            sql: include_str!("../migrations/008_printer_prompt_setting.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "replace printer_enabled and prompt with receipt_mode",
            sql: include_str!("../migrations/009_receipt_mode.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 10,
            description: "add ifields key setting for CNP payments",
            sql: include_str!("../migrations/010_ifields_key.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 11,
            description: "add logging/diagnostics settings",
            sql: include_str!("../migrations/011_logging.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 12,
            description: "add customer payment tokens table for card-on-file",
            sql: include_str!("../migrations/012_customer_payment_tokens.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 13,
            description: "add customer gateway sync columns and sync_log table",
            sql: include_str!("../migrations/013_customer_gateway_sync.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:lite-pos.db?mode=rwc&busy_timeout=5000", migrations)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            crypto::encrypt_value,
            crypto::decrypt_value,
            crypto::hash_pin,
            sola::process_sola_transaction,
            sola::cancel_sola_transaction,
            sola::build_sola_request_info,
            sola::process_sola_cnp_transaction,
            sola::sola_save_card,
            sola::process_sola_token_transaction,
            printing::get_system_printers,
            printing::print_receipt,
            logging::log_event,
            logging::get_log_entries,
            logging::get_log_dates,
            logging::export_log,
            logging::purge_old_logs,
            customer_sync::gateway_sync_acquire_lock,
            customer_sync::gateway_sync_release_lock,
            customer_sync::gateway_sync_is_running,
            customer_sync::gateway_list_customers,
            customer_sync::gateway_create_customer,
            customer_sync::gateway_update_customer,
            customer_sync::gateway_delete_customer
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Purge old log files on startup (default 30 days, errors are non-fatal)
            let handle = app.handle().clone();
            let _ = logging::purge_old_logs(handle, None);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

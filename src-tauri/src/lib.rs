mod booking_rules;
mod firestore_client;

use chrono::{Duration, NaiveDate};
use firestore_client::{new_shared_conn, ReservationDto, ReservationWrite, SharedConn};

#[tauri::command]
async fn database_connect(state: tauri::State<'_, SharedConn>, service_account_path: String) -> Result<String, String> {
	let mut conn = state.lock().await;
	conn
		.load_service_account_file(service_account_path.trim())
		.map_err(|e| e.to_string())?;
	let pid = conn.project_id().ok_or_else(|| "Missing project_id in JSON".to_string())?;
	let pid = pid.to_string();
	conn.ensure_token().await?;
	Ok(pid)
}

#[tauri::command]
async fn database_disconnect(state: tauri::State<'_, SharedConn>) -> Result<(), String> {
	let mut conn = state.lock().await;
	conn.clear();
	Ok(())
}

#[tauri::command]
async fn database_status(state: tauri::State<'_, SharedConn>) -> Result<bool, String> {
	let conn = state.lock().await;
	Ok(conn.is_connected())
}

#[tauri::command]
async fn fetch_reservations_week(
	state: tauri::State<'_, SharedConn>,
	week_start_iso: String,
) -> Result<Vec<ReservationDto>, String> {
	let week_end = week_end_inclusive(&week_start_iso)?;
	let mut conn = state.lock().await;
	conn
		.query_week(week_start_iso.trim(), &week_end)
		.await
}

#[tauri::command]
async fn reservation_create(
	state: tauri::State<'_, SharedConn>,
	payload: ReservationWrite,
) -> Result<String, String> {
	let mut conn = state.lock().await;
	conn.create_reservation(payload).await
}

#[tauri::command]
async fn reservation_update(
	state: tauri::State<'_, SharedConn>,
	id: String,
	payload: ReservationWrite,
) -> Result<(), String> {
	let mut conn = state.lock().await;
	conn.update_reservation(&id, payload).await
}

#[tauri::command]
async fn reservation_delete(state: tauri::State<'_, SharedConn>, id: String) -> Result<(), String> {
	let mut conn = state.lock().await;
	conn.delete_reservation(&id).await
}

fn week_end_inclusive(monday: &str) -> Result<String, String> {
	let d = NaiveDate::parse_from_str(monday, "%Y-%m-%d").map_err(|e| e.to_string())?;
	Ok((d + Duration::days(6)).format("%Y-%m-%d").to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.manage(new_shared_conn())
		.invoke_handler(tauri::generate_handler![
			database_connect,
			database_disconnect,
			database_status,
			fetch_reservations_week,
			reservation_create,
			reservation_update,
			reservation_delete
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

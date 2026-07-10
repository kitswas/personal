use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[serde(tag = "status")]
#[ts(export, export_to = "../../src/types/ipc_bindings.ts")]
pub enum IpcResponse<T, E> {
	Success { data: T },
	Error { error: E },
}

impl<T, E> From<Result<T, E>> for IpcResponse<T, E> {
	fn from(res: Result<T, E>) -> Self {
		match res {
			Ok(data) => IpcResponse::Success { data },
			Err(error) => IpcResponse::Error { error },
		}
	}
}

import { IpcResponse, AppError } from "./ipc_bindings";

export type { IpcResponse, AppError };

export type IpcState<T, E> =
	| { _tag: "Idle" }
	| { _tag: "Loading" }
	| IpcResponse<T, E>;

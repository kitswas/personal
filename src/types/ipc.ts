import type { AppError, IpcResponse } from "./ipc_bindings";

export type { AppError, IpcResponse };

export type IpcState<T, E> =
	| { _tag: "Idle" }
	| { _tag: "Loading" }
	| IpcResponse<T, E>;

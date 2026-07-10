import { createSignal } from "solid-js";
import type { IpcResponse, IpcState } from "../types/ipc";

export function createIpcCommand<Args extends any[], T, E>(
	commandFn: (...args: Args) => Promise<IpcResponse<T, E>>,
) {
	const [state, setState] = createSignal<IpcState<T, E>>({ _tag: "Idle" });

	const execute = async (...args: Args) => {
		setState({ _tag: "Loading" });
		try {
			const response = await commandFn(...args);
			setState(response);
		} catch (fatalError) {
			console.error("IPC Bridge Failure:", fatalError);
			setState({ _tag: "Idle" });
		}
	};

	return [state, execute, () => setState({ _tag: "Idle" })] as const;
}

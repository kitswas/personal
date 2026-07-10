// Auto-generated manually for now due to test linker errors
export type IpcResponse<T, E> =
	| { status: "Success"; data: T }
	| { status: "Error"; error: E };

export type AppError =
	| { type: "Db"; message: string }
	| { type: "WrongPassword"; message: string }
	| { type: "UnbalancedTransaction"; message: string; sum: number }
	| { type: "AccountNotFound"; message: string; id: string }
	| { type: "TransactionNotFound"; message: string; id: string }
	| { type: "TemplateParse"; message: string }
	| { type: "ImportParse"; message: string }
	| { type: "Keyring"; message: string }
	| { type: "Io"; message: string }
	| { type: "Other"; message: string };

export interface Account {
	id: string;
	name: string;
	accountType: string;
	commodity: string;
}

export interface Posting {
	id: string;
	transactionId: string;
	accountId: string;
	amount: number;
	commodity: string;
}

export interface Transaction {
	id: string;
	date: string;
	payee: string;
	notes: string;
}

export interface TransactionWithPostings {
	transaction: Transaction;
	postings: Posting[];
}

export interface PostingInput {
	accountId: string;
	amount: number;
	commodity: string;
}

export interface BalanceEntry {
	date: string;
	balance: number;
}

export type ParsedRow =
	| {
			status: "valid";
			rowIdx: number;
			date: string;
			payee: string;
			amount: number;
			commodity: string;
			suggestedAccountId: string;
			confidence: number;
	  }
	| {
			status: "invalid";
			rowIdx: number;
			rawData: string;
			errorReason: string;
	  };

export interface ValidRow {
	date: string;
	payee: string;
	amount: number;
	commodity: string;
	accountId: string;
	offsetAccountId: string;
}

export interface BatchResult {
	committed: number;
	failed: number;
}

export interface TemplateMeta {
	name: string;
	description: string;
	institution: string;
}

export const registerMainThreadArktsCallback: (callback: (message: string) => string) => void;
export const callMainThreadArktsCallback: (message: string) => string;
export const registerAsyncThreadArktsCallback: (callback: (message: string) => void) => void;
export const callAsyncThreadArktsCallback: (message: string) => void;
export const registerMainThreadDispatcher: (callback: (message: string) => string) => void;

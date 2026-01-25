export type SetData<T> = (key: keyof T, value: T[keyof T]) => void;

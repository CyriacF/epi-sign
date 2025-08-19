import { writable } from "svelte/store";
import type { User } from "./types";

export const isAuthenticated = writable<boolean>(false);
export const currentUser = writable<User | null>(null);

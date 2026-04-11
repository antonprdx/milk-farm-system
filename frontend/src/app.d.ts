declare global {
	namespace App {
		interface Locals {
			authenticated: boolean;
			role: string | null;
			mustChangePassword: boolean;
		}
	}
}

export {};

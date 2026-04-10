import { validate, type ValidationRule } from '$lib/utils/validators';

export type FieldValidation = {
	value: string | number | undefined;
	rules: ValidationRule[];
};

export function useFormValidation() {
	let errors = $state<Record<string, string>>({});

	function validateField(
		name: string,
		value: string | number | undefined,
		rules: ValidationRule[],
	): boolean {
		const err = validate(value, rules);
		if (err) {
			errors[name] = err;
			return false;
		}
		delete errors[name];
		return true;
	}

	function validateAll(fields: Record<string, FieldValidation>): boolean {
		errors = {};
		let valid = true;
		for (const [name, field] of Object.entries(fields)) {
			const err = validate(field.value, field.rules);
			if (err) {
				errors[name] = err;
				valid = false;
			}
		}
		return valid;
	}

	function clearField(name: string) {
		delete errors[name];
	}

	function clear() {
		errors = {};
	}

	function getError(name: string): string {
		return errors[name] || '';
	}

	return {
		get errors() {
			return errors;
		},
		validateField,
		validateAll,
		clearField,
		clear,
		getError,
	};
}

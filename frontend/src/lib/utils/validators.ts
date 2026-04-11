export type ValidationRule = {
	required?: boolean;
	minLength?: number;
	maxLength?: number;
	min?: number;
	max?: number;
	pattern?: RegExp;
	message?: string;
	custom?: (value: string | number | undefined) => string | undefined;
};

export function validate(
	value: string | number | undefined,
	rules: ValidationRule[],
): string | undefined {
	for (const rule of rules) {
		if (rule.custom) {
			const err = rule.custom(value);
			if (err) return err;
		}

		const str = value == null ? '' : String(value);

		if (rule.required && str.trim() === '') {
			return rule.message || 'Поле обязательно для заполнения';
		}

		if (rule.minLength != null && str.length < rule.minLength) {
			return rule.message || `Минимум ${rule.minLength} символов`;
		}

		if (rule.maxLength != null && str.length > rule.maxLength) {
			return rule.message || `Максимум ${rule.maxLength} символов`;
		}

		if (rule.min != null && Number(value) < rule.min) {
			return rule.message || `Минимальное значение: ${rule.min}`;
		}

		if (rule.max != null && Number(value) > rule.max) {
			return rule.message || `Максимальное значение: ${rule.max}`;
		}

		if (rule.pattern && !rule.pattern.test(str)) {
			return rule.message || 'Некорректный формат';
		}
	}

	return undefined;
}

export function validateForm(
	fields: { name: string; value: string | number | undefined; rules: ValidationRule[] }[],
): Record<string, string> {
	const errors: Record<string, string> = {};
	for (const field of fields) {
		const err = validate(field.value, field.rules);
		if (err) {
			errors[field.name] = err;
		}
	}
	return errors;
}

export const rules = {
	required: (message?: string): ValidationRule => ({
		required: true,
		message: message || 'Поле обязательно для заполнения',
	}),
	minLength: (len: number, message?: string): ValidationRule => ({
		minLength: len,
		message: message || `Минимум ${len} символов`,
	}),
	maxLength: (len: number, message?: string): ValidationRule => ({
		maxLength: len,
		message: message || `Максимум ${len} символов`,
	}),
	min: (val: number, message?: string): ValidationRule => ({
		min: val,
		message: message || `Минимальное значение: ${val}`,
	}),
	max: (val: number, message?: string): ValidationRule => ({
		max: val,
		message: message || `Максимальное значение: ${val}`,
	}),
	positive: (message?: string): ValidationRule => ({
		min: 0.001,
		message: message || 'Значение должно быть положительным',
	}),
	nonNegative: (message?: string): ValidationRule => ({
		min: 0,
		message: message || 'Значение не может быть отрицательным',
	}),
	percentage: (message?: string): ValidationRule => ({
		min: 0,
		max: 100,
		message: message || 'Значение должно быть от 0 до 100',
	}),
	email: (message?: string): ValidationRule => ({
		pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
		message: message || 'Некорректный email',
	}),
	dateNotFuture: (message?: string): ValidationRule => ({
		custom: (value) => {
			if (!value) return undefined;
			const d = new Date(String(value));
			if (d > new Date()) return message || 'Дата не может быть в будущем';
			return undefined;
		},
	}),
};

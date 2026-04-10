import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import FormField from '$lib/components/ui/FormField.svelte';

describe('FormField', () => {
	it('renders text input with label', () => {
		render(FormField, { props: { label: 'Имя', id: 'name', type: 'text' } });
		expect(screen.getByText('Имя')).toBeInTheDocument();
		expect(screen.getByLabelText('Имя')).toBeInTheDocument();
	});

	it('shows required asterisk when required', () => {
		render(FormField, { props: { label: 'Email', id: 'email', required: true } });
		expect(screen.getByText('Email *')).toBeInTheDocument();
	});

	it('renders select input with options', () => {
		const options = [
			{ value: 'male', label: 'Мужской' },
			{ value: 'female', label: 'Женский' },
		];
		render(FormField, { props: { label: 'Пол', id: 'gender', type: 'select', options } });
		expect(screen.getByText('Женский')).toBeInTheDocument();
		expect(screen.getByText('Мужской')).toBeInTheDocument();
	});

	it('renders checkbox', () => {
		render(FormField, { props: { label: 'Активный', type: 'checkbox' } });
		expect(screen.getByText('Активный')).toBeInTheDocument();
		expect(screen.getByRole('checkbox')).toBeInTheDocument();
	});

	it('renders textarea', () => {
		render(FormField, { props: { label: 'Описание', id: 'desc', type: 'textarea' } });
		expect(screen.getByLabelText('Описание')).toBeInTheDocument();
	});

	it('shows error message when error prop is set', () => {
		render(FormField, { props: { label: 'Логин', id: 'login', error: 'Обязательное поле' } });
		expect(screen.getByText('Обязательное поле')).toBeInTheDocument();
	});

	it('does not show error message when error is empty', () => {
		render(FormField, { props: { label: 'Логин', id: 'login' } });
		expect(screen.queryByText('Обязательное поле')).not.toBeInTheDocument();
	});
});

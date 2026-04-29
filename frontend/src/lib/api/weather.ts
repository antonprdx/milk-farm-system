import { api, buildQuery } from './client';

export interface WeatherData {
	date: string;
	temp_c: number | null;
	humidity: number | null;
	precipitation_mm: number | null;
	wind_speed: number | null;
	weather_main: string | null;
	weather_icon: string | null;
}

export function getCurrentWeather() {
	return api<{ data: WeatherData[] }>('/weather');
}

export function getWeatherForecast() {
	return api<{ data: WeatherData[] }>('/weather/forecast');
}

export function weatherIconUrl(icon: string | null) {
	if (!icon) return '';
	return `https://openweathermap.org/img/wn/${icon}@2x.png`;
}

export const WEATHER_LABELS: Record<string, string> = {
	Clear: 'Ясно',
	Clouds: 'Облачно',
	Rain: 'Дождь',
	Drizzle: 'Морось',
	Thunderstorm: 'Гроза',
	Snow: 'Снег',
	Mist: 'Туман',
	Fog: 'Туман',
	Haze: 'Дымка',
};

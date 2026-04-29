use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WeatherData {
    pub date: String,
    pub temp_c: Option<f64>,
    pub humidity: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed: Option<f64>,
    pub weather_main: Option<String>,
    pub weather_icon: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OwmCurrent {
    main: OwmMain,
    weather: Vec<OwmWeather>,
    wind: Option<OwmWind>,
    rain: Option<OwmRain>,
}

#[derive(Debug, serde::Deserialize)]
struct OwmMain {
    temp: f64,
    humidity: f64,
}

#[derive(Debug, serde::Deserialize)]
struct OwmWeather {
    main: String,
    icon: String,
}

#[derive(Debug, serde::Deserialize)]
struct OwmWind {
    speed: f64,
}

#[derive(Debug, serde::Deserialize)]
struct OwmRain {
    #[serde(rename = "1h")]
    one_hour: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
struct OwmForecastItem {
    dt: i64,
    main: OwmMain,
    weather: Vec<OwmWeather>,
    wind: Option<OwmWind>,
    rain: Option<OwmRain>,
}

#[derive(Debug, serde::Deserialize)]
struct OwmForecastResponse {
    list: Vec<OwmForecastItem>,
}

pub async fn get_current_weather(pool: &PgPool) -> Result<Vec<WeatherData>, AppError> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let cached: Option<(String, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT date::text, temp_c, humidity, precipitation_mm, wind_speed, weather_main, weather_icon FROM weather_cache WHERE date = CURRENT_DATE",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if let Some((date, temp, hum, prec, wind, main, icon)) = cached {
        return Ok(vec![WeatherData {
            date,
            temp_c: temp,
            humidity: hum,
            precipitation_mm: prec,
            wind_speed: wind,
            weather_main: main,
            weather_icon: icon,
        }]);
    }

    let api_key = crate::services::system_settings_service::get_value(pool, "weather_api_key")
        .await
        .unwrap_or_default();

    if api_key.is_empty() {
        return Ok(vec![]);
    }

    let lat = crate::services::system_settings_service::get_value(pool, "weather_lat")
        .await
        .unwrap_or_else(|_| "55.75".into());
    let lon = crate::services::system_settings_service::get_value(pool, "weather_lon")
        .await
        .unwrap_or_else(|_| "37.62".into());

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}",
        lat, lon, api_key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<OwmCurrent>().await {
                let precipitation = data.rain.and_then(|r| r.one_hour).unwrap_or(0.0);
                let wind = data.wind.map(|w| w.speed);
                let main_weather = data.weather.first().map(|w| w.main.clone());
                let icon = data.weather.first().map(|w| w.icon.clone());

                sqlx::query(
                    "INSERT INTO weather_cache (date, temp_c, humidity, precipitation_mm, wind_speed, weather_main, weather_icon)
                     VALUES (CURRENT_DATE, $1, $2, $3, $4, $5, $6)
                     ON CONFLICT (date) DO UPDATE SET temp_c=$1, humidity=$2, precipitation_mm=$3, wind_speed=$4, weather_main=$5, weather_icon=$6, fetched_at=NOW()",
                )
                .bind(data.main.temp)
                .bind(data.main.humidity)
                .bind(precipitation)
                .bind(wind)
                .bind(&main_weather)
                .bind(&icon)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;

                Ok(vec![WeatherData {
                    date: today,
                    temp_c: Some(data.main.temp),
                    humidity: Some(data.main.humidity),
                    precipitation_mm: Some(precipitation),
                    wind_speed: wind,
                    weather_main: main_weather,
                    weather_icon: icon,
                }])
            } else {
                Ok(vec![])
            }
        }
        _ => Ok(vec![]),
    }
}

pub async fn get_forecast(pool: &PgPool) -> Result<Vec<WeatherData>, AppError> {
    let api_key = crate::services::system_settings_service::get_value(pool, "weather_api_key")
        .await
        .unwrap_or_default();

    if api_key.is_empty() {
        return Ok(vec![]);
    }

    let lat = crate::services::system_settings_service::get_value(pool, "weather_lat")
        .await
        .unwrap_or_else(|_| "55.75".into());
    let lon = crate::services::system_settings_service::get_value(pool, "weather_lon")
        .await
        .unwrap_or_else(|_| "37.62".into());

    let url = format!(
        "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&units=metric&appid={}",
        lat, lon, api_key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<OwmForecastResponse>().await {
                let mut result = Vec::new();
                let mut seen_dates = std::collections::HashSet::new();

                for item in data.list.into_iter() {
                    let date = chrono::DateTime::from_timestamp(item.dt, 0)
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();

                    if seen_dates.contains(&date) || date.is_empty() {
                        continue;
                    }
                    if seen_dates.len() >= 5 {
                        break;
                    }
                    seen_dates.insert(date.clone());

                    let precipitation = item.rain.and_then(|r| r.one_hour).unwrap_or(0.0);
                    let main_weather = item.weather.first().map(|w| w.main.clone());
                    let icon = item.weather.first().map(|w| w.icon.clone());

                    result.push(WeatherData {
                        date,
                        temp_c: Some(item.main.temp),
                        humidity: Some(item.main.humidity),
                        precipitation_mm: Some(precipitation),
                        wind_speed: item.wind.map(|w| w.speed),
                        weather_main: main_weather,
                        weather_icon: icon,
                    });
                }

                Ok(result)
            } else {
                Ok(vec![])
            }
        }
        _ => Ok(vec![]),
    }
}

pub async fn get_profitability_settings(pool: &PgPool) -> Result<(f64, f64), AppError> {
    let milk_price = crate::services::system_settings_service::get_value(pool, "milk_price_per_liter")
        .await
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30.0);
    let feed_cost = crate::services::system_settings_service::get_value(pool, "feed_cost_per_kg")
        .await
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15.0);
    Ok((milk_price, feed_cost))
}

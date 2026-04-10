use crate::errors::AppError;

pub fn required_non_empty(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' не может быть пустым",
            field
        )));
    }
    Ok(())
}

pub fn max_len(value: &str, max: usize, field: &str) -> Result<(), AppError> {
    if value.len() > max {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' слишком длинное (максимум {} символов)",
            field, max
        )));
    }
    Ok(())
}

pub fn opt_max_len(value: &Option<String>, max: usize, field: &str) -> Result<(), AppError> {
    if let Some(v) = value {
        max_len(v, max, field)?;
    }
    Ok(())
}

pub fn positive_i32(value: i32, field: &str) -> Result<(), AppError> {
    if value <= 0 {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' должно быть положительным числом",
            field
        )));
    }
    Ok(())
}

pub fn opt_positive_i32(value: &Option<i32>, field: &str) -> Result<(), AppError> {
    if let Some(v) = value {
        positive_i32(*v, field)?;
    }
    Ok(())
}

pub fn non_negative_f64(value: f64, field: &str) -> Result<(), AppError> {
    if value < 0.0 {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' не может быть отрицательным",
            field
        )));
    }
    Ok(())
}

pub fn opt_non_negative_f64(value: &Option<f64>, field: &str) -> Result<(), AppError> {
    if let Some(v) = value {
        non_negative_f64(*v, field)?;
    }
    Ok(())
}

pub fn percentage_f64(value: f64, field: &str) -> Result<(), AppError> {
    if value < 0.0 || value > 100.0 {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' должно быть от 0 до 100",
            field
        )));
    }
    Ok(())
}

pub fn opt_percentage_f64(value: &Option<f64>, field: &str) -> Result<(), AppError> {
    if let Some(v) = value {
        percentage_f64(*v, field)?;
    }
    Ok(())
}

pub fn positive_percentage_f64(value: f64, field: &str) -> Result<(), AppError> {
    if value <= 0.0 || value > 100.0 {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' должно быть больше 0 и не больше 100",
            field
        )));
    }
    Ok(())
}

pub fn username(value: &str) -> Result<(), AppError> {
    required_non_empty(value, "Имя пользователя")?;
    max_len(value, 50, "Имя пользователя")?;
    if value.trim().len() < 2 {
        return Err(AppError::BadRequest(
            "Имя пользователя должно быть не короче 2 символов".into(),
        ));
    }
    Ok(())
}

pub fn password(value: &str) -> Result<(), AppError> {
    required_non_empty(value, "Пароль")?;
    if value.len() < 8 {
        return Err(AppError::BadRequest(
            "Пароль должен быть не короче 8 символов".into(),
        ));
    }
    let has_letter = value.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = value.chars().any(|c| c.is_ascii_digit());
    if !has_letter || !has_digit {
        return Err(AppError::BadRequest(
            "Пароль должен содержать буквы и цифры".into(),
        ));
    }
    Ok(())
}

pub fn opt_email(value: &Option<String>) -> Result<(), AppError> {
    if let Some(v) = value {
        if !v.trim().is_empty() && !v.contains('@') {
            return Err(AppError::BadRequest("Некорректный email".into()));
        }
    }
    Ok(())
}

pub fn date_not_future(date: &chrono::NaiveDate, field: &str) -> Result<(), AppError> {
    let today = chrono::Utc::now().date_naive();
    if *date > today {
        return Err(AppError::BadRequest(format!(
            "Поле '{}' не может быть в будущем",
            field
        )));
    }
    Ok(())
}

pub fn opt_date_not_future(date: &Option<chrono::NaiveDate>, field: &str) -> Result<(), AppError> {
    if let Some(d) = date {
        date_not_future(d, field)?;
    }
    Ok(())
}

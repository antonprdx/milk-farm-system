# Математические модели и формулы — Молочная ферма

> Полный справочник всех алгоритмов, моделей и расчётов, применяемых в системе.

---

## Содержание

1. [ML-модели (Python)](#1-ml-модели-python)
2. [Прогнозирование надоя](#2-прогнозирование-надоя)
3. [Здоровье вымени — мастит](#3-здоровье-вымени--мастит)
4. [Метаболический статус — кетоз/ацидоз](#4-метаболический-статус--кетозацидоз)
5. [Воспроизводство — охота](#5-воспроизводство--охота)
6. [Выбраковка — culling risk](#6-выбраковка--culling-risk)
7. [Оценка упитанности — BCS](#7-оценка-упитанности--bcs)
8. [Кормление](#8-кормление)
9. [Кластеризация стада](#9-кластеризация-стада)
10. [Аномалии оборудования](#10-аномалии-оборудования)
11. [Аналитика стада (Rust)](#11-аналитика-стада-rust)
12. [Предиктивный сервис (Rust)](#12-предиктивный-сервис-rust)
13. [Дрифт-мониторинг (MLOps)](#13-дрифт-мониторинг-mlops)
14. [Оптимизация гиперпараметров](#14-оптимизация-гиперпараметров)

---

## 1. ML-модели (Python)

Все ML-модели обучаются в сервисе `analytics-ml` и деплоятся через ONNX или joblib.

| Модель | Алгоритм | Версия | Файл |
|--------|----------|--------|------|
| Прогноз надоя (животное) | XGBoost + Quantile Regression | xgboost-v2-quantile | `milk_forecast.py` |
| Прогноз надоя (стадо) | Facebook Prophet | prophet-v1 | `herd_milk_prophet.py` |
| Мастит | XGBoost Classifier | xgboost-v2 | `mastitis.py` |
| Кетоз | XGBoost Classifier | xgboost-v1 | `ketosis_warning.py` |
| Охота | XGBoost Classifier | xgboost-v1 | `estrus.py` |
| Выбраковка | XGBoost Regressor | xgboost-v1 | `culling.py` |
| BCS оценка | XGBoost Regressor | bcs-v1 | `bcs_estimator.py` |
| Кластеризация | KMeans (auto-k) | kmeans-v2 | `clustering.py` |
| Аномалии оборудования | Isolation Forest | isolation-forest-v1 | `equipment_anomaly.py` |

**Тюнинг гиперпараметров:** Optuna (TPE sampler), по 30 trials с timeout 120s. Если Optuna не установлен — дефолт: `n_estimators=100, max_depth=4, lr=0.1`.

---

## 2. Прогнозирование надоя

### 2.1 Рекуррентный прогноз (XGBoost, на животное)

**Признаки (9):**

| Признак | Формула |
|---------|---------|
| `milk_lag1` | `milk_amount.shift(1)` |
| `milk_lag7` | `milk_amount.shift(7)` |
| `milk_roll7` | `milk_amount.rolling(7).mean()` |
| `milk_roll30` | `milk_amount.rolling(30).mean()` |
| `milk_diff1` | `milk_amount.diff(1)` |
| `milk_diff7` | `milk_amount.diff(7)` |
| `feed_amount` | прогнозируемое значение (из feature_forecast) |
| `rumination` | прогнозируемое значение |
| `activity` | прогнозируемое значение |

**Quantile Regression (доверительные интервалы):**

Помимо основной модели обучаются два дополнительных XGBoost:
- **q10:** `objective="reg:quantileerror", quantile_alpha=0.1` — нижняя граница
- **q90:** `objective="reg:quantileerror", quantile_alpha=0.9` — верхняя граница

Если q10/q90 недоступны (старая модель), fallback:
```
std_est = |pred| × 0.1
lower   = pred − 1.96 × std_est
upper   = pred + 1.96 × std_est
```

**Файл:** `analytics-ml/app/models/milk_forecast.py`

### 2.2 Prophet (на стадо)

Facebook Prophet с параметрами: `yearly_seasonality=True`, `weekly_seasonality=True`, `daily_seasonality=False`, `uncertainty_samples=500`.

Prophet автоматически определяет тренд и сезонность через аддитивную декомпозицию:

```
y(t) = g(t) + s(t) + h(t) + ε(t)
```

где `g(t)` — тренд, `s(t)` — сезонность, `h(t)` — эффект праздников, `ε(t)` — шум.

**Тренд (%):**
```
trend_pct = ((mean(yhat[-7:]) − mean(yhat[-14:-7])) / mean(yhat[-14:-7])) × 100
```

**Классификация направления тренда:**
| trend_pct | Направление |
|-----------|-------------|
| > 5% | `significant_up` |
| > 2% | `up` |
| < −5% | `significant_down` |
| < −2% | `down` |
| иначе | `stable` |

**Файл:** `analytics-ml/app/models/herd_milk_prophet.py`

### 2.3 Прогноз feed/rumination/activity (для подачи в milk forecast)

Экспоненциальное сглаживание с недельной сезонностью:

```
level  = mean(values[-7:])
trend  = (mean(values[-7:]) − mean(values[-30:−23])) / 7.0
seasonality[d] = mean(values[-3:] для дня недели d) − mean(values)

pred(h) = level + (h+1) × trend × 0.3 + seasonality[(n+h) % 7]
```

Коэффициенты: `trend_damping = 0.3`, `seasonality_window = 3`.

**Файл:** `analytics-ml/app/services/feature_forecast.py`

---

## 3. Здоровье вымени — мастит

### 3.1 Генерация обучающих лейблов (Python)

```
label = 1 if:
  recent_scc > 300 000
  OR (recent_scc > 200 000 AND scc_trend_ratio > 1.5)
  OR (recent_scc > 150 000 AND avg_conductivity > 55)
  OR (milk_deviation < −0.15 AND recent_scc > 100 000)
  OR (cond_asymmetry > 5 AND recent_scc > 100 000)
  OR (0 < fat_protein_ratio < 1.0 AND recent_scc > 100 000)
```

### 3.2 ML-модель (XGBoost Classifier)

**Признаки (10):** `age_years`, `recent_scc`, `scc_trend_ratio`, `avg_conductivity`, `milk_deviation`, `dim_days`, `avg_rumination_7d`, `avg_activity_7d`, `fat_protein_ratio`, `cond_asymmetry`

### 3.3 Rule-based fallback (Rust)

```
score = 0
score += 0.40  if scc > 500 000
score += 0.25  if scc > 300 000
score += 0.10  if scc > 200 000
score += 0.25  if scc_trend > 2.0
score += 0.15  if scc_trend > 1.5
score += 0.20  if conductivity > 60
score += 0.15  if milk_deviation < −0.15
score += 0.10  if dim < 30
score = min(score, 1.0)
```

### 3.4 Уровни риска

| Вероятность | Уровень |
|-------------|---------|
| ≥ 0.6 | `high` |
| ≥ 0.3 | `medium` |
| < 0.3 | `low` |

**Файл:** `analytics-ml/app/models/mastitis.py`, `backend/src/services/predictive_service.rs`

---

## 4. Метаболический статус — кетоз/ацидоз

### 4.1 Fat-Protein Ratio (FPR)

```
FPR = fat_percentage / protein_percentage
```

### 4.2 Классификация по FPR

| FPR | Статус |
|-----|--------|
| < 1.0 | `ketosis_risk` |
| < 1.1 | `ketosis_warning` |
| 1.1–1.4 | `normal` |
| > 1.4 | `acidosis_warning` |
| > 1.5 | `acidosis_risk` |

### 4.3 Генерация обучающих лейблов

```
label = 1 if:
  fpr_7d > 1.5
  OR fpr_7d < 1.0
  OR (fpr_7d > 1.4 AND rumination_trend < −0.1)
  OR (fpr_7d < 1.1 AND dim_days < 60)
```

### 4.4 ML-модель (XGBoost Classifier)

**Признаки (10):** `fpr_7d`, `fpr_14d`, `fpr_trend`, `avg_rumination_7d`, `avg_rumination_14d`, `rumination_trend`, `avg_milk_7d`, `milk_trend`, `dim_days`, `lactation_number`

**Файл:** `analytics-ml/app/models/ketosis_warning.py`

---

## 5. Воспроизводство — охота

### 5.1 Генерация обучающих лейблов

```
label = 1 if:
  activity_ratio_7d > 1.4
  OR (activity_ratio_7d > 1.2 AND rumination_ratio_7d < 0.85)
  OR (activity_ratio_7d > 1.15 AND 40 ≤ dim_days ≤ 120)
```

### 5.2 ML-модель (XGBoost Classifier)

**Признаки (8):** `activity_ratio_7d`, `rumination_ratio_7d`, `milk_ratio_7d`, `dim_days`, `lactation_number`, `days_since_last_heat`, `avg_activity_14d`, `avg_rumination_14d`

### 5.3 Rule-based fallback (Rust)

```
score = 0
score += 0.45  if activity_ratio > 1.4
score += 0.25  if activity_ratio > 1.2
score += 0.35  if rumination_ratio < 0.8
score += 0.15  if rumination_ratio < 0.9
score += 0.20  if milk_ratio < 0.85
score += 0.10  if dim ∈ [30, 150]
score ×= 0.5   if outside dim window
score = min(score, 1.0)
```

### 5.4 Статусы

| Вероятность | Статус |
|-------------|--------|
| ≥ 0.7 | `in_heat` / `in_estrus` |
| ≥ 0.4 | `approaching` |
| < 0.4 | `not_in_heat` / `possible` |

### 5.5 Фертильное окно (Rust)

```
in_window = 30 ≤ dim ≤ 150
score += 40  if activity_signal > 1.3
score += 30  if rumination_signal < 0.85
score += 20  if milk_signal < 0.9
score += 10  if in_window

status: ≥60 "optimal", ≥30 "approaching", in_window "in_window", else "outside_window"
```

### 5.6 Биологические константы

| Параметр | Значение |
|----------|----------|
| Период охоты | 21 день |
| Срок стельности | 283 дня (прогноз) / 280 (запуск) |
| Запуск до отёла | 60 дней |

**Файл:** `analytics-ml/app/models/estrus.py`, `backend/src/services/predictive_service.rs`

---

## 6. Выбраковка — culling risk

### 6.1 Формула риска (Python, обучающий таргет)

```
risk = 0
risk += 0.40  if age ≥ 10
risk += 0.25  if 8 ≤ age < 10
risk += 0.10  if 6 ≤ age < 8
risk += 0.30  if avg_milk_30d < 15
risk += 0.10  if 15 ≤ avg_milk_30d < 20
risk += 0.25  if avg_scc_90d > 300 000
risk += 0.10  if 200 000 < avg_scc_90d ≤ 300 000
risk += 0.20  if calving_interval > 450
risk += 0.10  if 400 < calving_interval ≤ 450
risk += 0.10  if lactation_count ≥ 6
risk = min(risk, 1.0)
expected_days = 730 × (1 − risk)
```

### 6.2 Обратный расчёт вероятности

```
risk_probability = 1.0 − min(expected_days / 730, 1.0)
```

### 6.3 ML-модель (XGBoost Regressor)

**Признаки (9):** `age_years`, `avg_milk_30d`, `avg_scc_90d`, `calving_interval`, `lactation_count`, `avg_rumination_30d`, `avg_milk_7d`, `avg_activity_30d`, `current_dim`

**Файл:** `analytics-ml/app/models/culling.py`

---

## 7. Оценка упитанности — BCS

### 7.1 Формульная оценка

Шкала: 1.0 (истощение) — 5.0 (ожирение).

```
bcs = 3.0  (базовое значение)

# По весу (если есть):
bcs += (weight − 550) / 200.0 × 0.5

# По надою (если нет веса):
bcs −= 0.3  if milk > 35
bcs −= 0.1  if milk > 25
bcs += 0.2  if milk < 15

# По дню лактации:
bcs −= 0.4  if dim < 60
bcs −= 0.2  if dim < 100
bcs += 0.3  if dim > 250

# По жвачке:
bcs −= 0.2  if rumination < 400
bcs += 0.1  if rumination > 550

# По эффективности корма:
ratio = milk / feed
bcs −= 0.2  if ratio > 1.8
bcs += 0.2  if ratio < 1.0

bcs = clamp(bcs, 1.0, 5.0)
```

### 7.2 Классификация

| BCS | Статус |
|-----|--------|
| < 2.5 | `underconditioned` |
| 2.5–3.75 | `optimal` |
| > 3.75 | `overconditioned` |

**Файл:** `analytics-ml/app/models/bcs_estimator.py`

---

## 8. Кормление

### 8.1 Рекомендация по корму (Python — основная)

```
base = milk × 0.45 + rumination × 0.005 + min(max(dim, 0), 100) × 0.02 + 5.0

DIM-фактор:       Лактация-фактор:
dim < 60:   1.15  lac ≤ 1:  1.00
dim < 120:  1.05  lac ≤ 3:  1.05
dim > 250:  0.90  lac > 3:  1.10
иначе:      1.00

recommended = base × DIM_factor × lactation_factor
```

### 8.2 Рекомендация по корму (Rust — fallback)

```
base = 12.0 + milk × 0.4

# Те же DIM и lactation факторы
recommended = base × DIM_factor × lactation_factor

diff = recommended − current
"increase" if diff > 2.0, "reduce" if diff < −2.0, else "maintain"
```

### 8.3 Эффективность корма

```
feed_efficiency = avg_daily_milk / avg_daily_feed
feed_cost_ratio = feed_cost / milk_revenue
```

**Файл:** `analytics-ml/app/models/feed_recommendation.py`, `backend/src/services/predictive_service.rs`

---

## 9. Кластеризация стада

### 9.1 KMeans с автоматическим выбором K

**Признаки (8):** `avg_milk`, `std_milk`, `avg_rumination`, `avg_activity`, `avg_feed`, `milk_cv`, `dim_days`, `lactation_number`

```
milk_cv = std_milk / avg_milk    (коэффициент вариации)
```

**Предобработка:** StandardScaler (z-score нормализация).

**Выбор K:** перебор от 2 до 8, выбор K с максимальным `silhouette_score` (sample_size ≤ 5000).

**Расстояние до центра кластера:** евклидово расстояние в масштабированном пространстве.

### 9.2 Быстрая кластеризация (Rust, single-animal)

```
if milk > 30 AND rumination > 500 → "High producing"
elif milk > 25                     → "Average producing"
elif dim < 100                     → "Fresh cows"
else                               → "Low producing"

distance = √((milk−25)² + (rumination−450)² + (dim−150)² + (lac−2)²)
```

**Файл:** `analytics-ml/app/models/clustering.py`, `backend/src/services/predictive_service.rs`

---

## 10. Аномалии оборудования

### Isolation Forest

**Признаки (7):** `avg_conductivity`, `max_quarter_asymmetry`, `avg_milk_temperature`, `std_milk_temperature`, `avg_milk_yield_per_visit`, `avg_milk_speed`, `anomaly_rate_7d`

Параметры: `n_estimators=100`, `contamination=0.05`.

**Серьёзность:**
| Score | Severity |
|-------|----------|
| < −0.3 | `critical` |
| < −0.1 | `warning` |
| иначе | `normal` |

**Файл:** `analytics-ml/app/models/equipment_anomaly.py`

---

## 11. Аналитика стада (Rust)

### 11.1 Временные ряды — Holt-Winters

**Двойное экспоненциальное сглаживание (Holt):**

```
level₀  = y₀
trend₀  = y₁ − y₀

level_t  = α × y_t + (1−α) × (level_{t−1} + trend_{t−1})
trend_t  = β × (level_t − level_{t−1}) + (1−β) × trend_{t−1}
```

**Тройное экспоненциальное сглаживание (Holt-Winters), period=7:**

```
level_t    = α × (y_t − s_{t−p}) + (1−α) × (level_{t−1} + trend_{t−1})
trend_t    = β × (level_t − level_{t−1}) + (1−β) × trend_{t−1}
season_t   = γ × (y_t − level_t) + (1−γ) × s_{t−p}
```

**Оптимизация параметров:** grid search по validation SSE.

| Модель | α | β | γ |
|--------|---|---|---|
| Holt | 0.1–0.9 (шаг 0.1) | 0.05–0.25 (шаг 0.05) | — |
| Holt-Winters | 0.1–0.9 | 0.05–0.25 | 0.05–0.25 |

Дефолт: α=0.3, β=0.1, γ=0.1.

### 11.2 Прогноз с доверительным интервалом

```
z = 1.96   (95% CI)

pred_h  = level + h × trend + seasonal[h % 7]
error_h = z × RMSE × √(1 + h × 0.1)

lower = pred − error
upper = pred + error
```

### 11.3 Очистка данных

**IQR-фильтр выбросов:**
```
Q1 = sorted[n/4]
Q3 = sorted[3n/4]
IQR = Q3 − Q1
outlier if value < Q1 − 1.5×IQR  OR  value > Q3 + 1.5×IQR
```

**Интерполяция NaN:**
```
value[i] = value[l] × (1−t) + value[r] × t
where t = (i−l) / (r−l)
```

### 11.4 Структурные разломы

T-тест на скользящем окне (window=7):

```
t_stat = |after_mean − before_mean| / (pooled_std × √(2/window))
```

Разлом если `t_stat > 3.0` и расстояние от предыдущего > window.

### 11.5 Метрики качества

```
RMSE = √(Σ(residual²) / n)
MAPE = (Σ|actual − fitted| / |actual|) / n × 100%
```

### 11.6 KPI стада

| KPI | Формула |
|-----|---------|
| Концепшн-рейт | `(pregnancies / inseminations) × 100%` (12 мес.) |
| Эффективность корма | `Σmilk_30d / Σfeed_30d` |
| Процент отказов | `(Σrefusals / Σmilkings) × 100%` (90 дней) |
| Сезонный индекс | `avg_daily_milk_month / overall_avg_daily_milk` |

**Файл:** `backend/src/services/analytics_service.rs`

---

## 12. Предиктивный сервис (Rust)

### 12.1 Кривая лактации Вуда (Wood's model)

```
Y(t) = a × t^b × exp(−c × t)
```

**Лог-линеаризация для OLS:**
```
ln(Y) = ln(a) + b × ln(t) + (−c) × t
```

Решается система нормальных уравнений 3×3 методом Гаусса с частичным выбором главного элемента.

**Ограничения:** `c ∈ [0.001, 0.1]`, минимум 5 точек. Дефолт: `a=20.0, b=0.2, c=0.003`.

**305-дневный прогноз:**
```
predicted_305 = Σ(d=1..305) Y(d)
```

### 12.2 Health Index

```
score = 100.0  (старт)

# Молоко (z-score):
z_milk = (short_avg − long_avg) / stddev
score −= 30  if z_milk < −2.0
score −= 15  if z_milk < −1.0

# Жвачка (z-score):
score −= 25  if z_rum < −2.0
score −= 10  if z_rum < −1.0

# Активность (z-score):
score −= 20  if z_act < −2.0
score −= 10  if z_act < −1.0

# SCC:
scc_ratio = recent_scc / baseline_scc
z_scc = (recent_scc − baseline_scc) / (baseline_scc × 0.5)
score −= 25  if z_scc > 2.0
score −= 10  if z_scc > 1.0 OR scc_ratio > 1.5

score = clamp(score, 0, 100)
```

**Уровни:**
| Score | Риск |
|-------|------|
| < 40 | `critical` |
| < 60 | `high` |
| < 80 | `moderate` |
| ≥ 80 | `low` |

### 12.3 Асимметрия долей вымени

```
avg = (LF + LR + RF + RR) / 4
max_asymmetry = max(|LF−avg|, |LR−avg|, |RF−avg|, |RR−avg|)

risk:
  > 10.0  → "high"
  > 5.0   → "medium"
  avg>55  → "elevated"
  иначе   → "low"
```

### 12.4 Рентабельность

```
milk_revenue_day = avg_daily_milk × price_per_liter
feed_cost_day    = avg_daily_feed × cost_per_kg
margin_day       = milk_revenue_day − feed_cost_day
margin_30d       = margin_day × 30
feed_cost_ratio  = feed_cost / milk_revenue
```

### 12.5 Lifetime Value

```
milk_price = 25.0 ₽/л
feed_cost  = 150.0 ₽/день
lactation_days = 305

remaining_lactations = max(6 − lactation_count, 0)
projected_milk_value = remaining × avg_milk_per_lac × 25.0
projected_feed_cost  = remaining × 305 × 150.0
net_value = projected_milk_value − projected_feed_cost

recommendation:
  "culling_candidate" if remaining ≤ 0 OR age > 8
  "review"            if net < 0
  "last_lactation"    if remaining ≤ 1
  "keep"              otherwise
```

### 12.6 Dry-off оптимизатор

```
expected_calving = last_insemination_date + 280 дней
dry_off_date     = expected_calving − 60 дней
days_until       = dry_off_date − today

readiness:
  ≤ 0  → "overdue"
  ≤ 7  → "now"
  ≤ 21 → "soon"
  else → "monitor"
```

### 12.7 Энергетический баланс (FPR)

```
FPR = fat_pct / protein_pct
FPR_trend = (recent_FPR / baseline_FPR) − 1
```

### 12.8 Alert-пороги (конфигурируемые)

| Параметр | Дефолт | Формула |
|----------|--------|---------|
| milk_drop_factor | 5% | `1.0 − min(alert_min_milk/100, 0.99)` |
| scc_multiplier | 400k | `alert_max_scc / 200.0` |
| activity_drop_factor | 30% | `1.0 − min(drop_pct/100, 0.99)` |
| rumination_drop | 25% | `rum < baseline × 0.75` |

**Файл:** `backend/src/services/predictive_service.rs`

---

## 13. Дрифт-мониторинг (MLOps)

### 13.1 Prediction Drift

```
z_score = |recent_mean − baseline_mean| / baseline_std
drift_detected = z_score > 2.0
```

- recent: последние 100 батчей предсказаний
- baseline: первая половина всех записей
- Минимум 10 батчей для анализа

### 13.2 Feature Drift

Для каждого числового признака записываются: mean, std, min, max, Q25, Q75.

```
z_score = |recent_feature_mean − baseline_feature_mean| / baseline_feature_std
drift_detected = z_score > 2.5
```

**Файл:** `analytics-ml/app/services/drift_monitor.py`

---

## 14. Оптимизация гиперпараметров

### Optuna TPE Sampler

**Пространство поиска (общее для всех XGBoost):**

| Гиперпараметр | Диапазон | Масштаб |
|---------------|----------|---------|
| `n_estimators` | [50, 300] | линейный |
| `max_depth` | [2, 8] | линейный |
| `learning_rate` | [0.01, 0.3] | логарифмический |
| `subsample` | [0.6, 1.0] | линейный |
| `colsample_bytree` | [0.6, 1.0] | линейный |
| `min_child_weight` | [1, 10] | линейный |
| `reg_alpha` (L1) | [1e-8, 10.0] | логарифмический |
| `reg_lambda` (L2) | [1e-8, 10.0] | логарифмический |

**Целевые метрики:** `roc_auc` (классификация), `neg_mean_absolute_error` (регрессия).

**CV:** `min(5, max(2, n_samples // 30))`.

**Файл:** `analytics-ml/app/services/hyperopt.py`

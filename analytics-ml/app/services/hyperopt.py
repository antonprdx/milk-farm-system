from __future__ import annotations

import logging

import numpy as np

logger = logging.getLogger(__name__)

_OPTUNA_AVAILABLE = False
try:
    import optuna
    optuna.logging.set_verbosity(optuna.logging.WARNING)
    _OPTUNA_AVAILABLE = True
except ImportError:
    pass

_LIGHTGBM_AVAILABLE = False
try:
    from lightgbm import LGBMClassifier, LGBMRegressor
    _LIGHTGBM_AVAILABLE = True
except ImportError:
    pass

_CATBOOST_AVAILABLE = False
try:
    from catboost import CatBoostClassifier, CatBoostRegressor
    _CATBOOST_AVAILABLE = True
except ImportError:
    pass

_AVAILABLE_BACKENDS = ["xgboost"]
if _LIGHTGBM_AVAILABLE:
    _AVAILABLE_BACKENDS.append("lightgbm")
if _CATBOOST_AVAILABLE:
    _AVAILABLE_BACKENDS.append("catboost")


def is_available() -> bool:
    return _OPTUNA_AVAILABLE


def available_backends() -> list[str]:
    return list(_AVAILABLE_BACKENDS)


def tune_classifier(
    X: np.ndarray,
    y: np.ndarray,
    backends: list[str] | None = None,
    n_trials: int = 30,
    timeout: int = 120,
    scale_pos_weight: float | None = None,
) -> tuple[dict, str]:
    if not _OPTUNA_AVAILABLE:
        return _default_xgb_classifier_params(), "xgboost"

    if backends is None:
        backends = _AVAILABLE_BACKENDS

    from sklearn.model_selection import cross_val_score

    best_score = -np.inf
    best_params = None
    best_backend = "xgboost"

    for backend in backends:
        if backend not in _AVAILABLE_BACKENDS:
            continue

        def _make_objective(b):
            def objective(trial):
                params, clf_cls = _sample_classifier_params(trial, b)
                if scale_pos_weight is not None:
                    params["scale_pos_weight"] = scale_pos_weight
                clf = clf_cls(**params)
                cv = min(5, max(2, len(X) // 30))
                scores = cross_val_score(
                    clf, X, y, cv=cv, scoring="roc_auc",
                    n_jobs=-1,
                )
                return scores.mean()
            return objective

        try:
            study = optuna.create_study(direction="maximize")
            per_backend_trials = max(n_trials // len(backends), 10)
            study.optimize(_make_objective(backend), n_trials=per_backend_trials, timeout=timeout // max(len(backends), 1))

            if study.best_value > best_score:
                best_score = study.best_value
                best_backend = backend
                best_params = dict(study.best_params)
        except Exception as e:
            logger.warning("Backend %s tuning failed: %s", backend, e)
            continue

    if best_params is None:
        return _default_xgb_classifier_params(), "xgboost"

    params = _finalize_params(best_params, best_backend, "classifier")
    logger.info("Best backend=%s, AUC=%.4f, params=%s", best_backend, best_score, params)
    return params, best_backend


def tune_regressor(
    X: np.ndarray,
    y: np.ndarray,
    backends: list[str] | None = None,
    n_trials: int = 30,
    timeout: int = 120,
) -> tuple[dict, str]:
    if not _OPTUNA_AVAILABLE:
        return _default_xgb_regressor_params(), "xgboost"

    if backends is None:
        backends = _AVAILABLE_BACKENDS

    from sklearn.model_selection import cross_val_score

    best_score = -np.inf
    best_params = None
    best_backend = "xgboost"

    for backend in backends:
        if backend not in _AVAILABLE_BACKENDS:
            continue

        def _make_objective(b):
            def objective(trial):
                params, reg_cls = _sample_regressor_params(trial, b)
                reg = reg_cls(**params)
                cv = min(5, max(2, len(X) // 30))
                scores = cross_val_score(
                    reg, X, y, cv=cv, scoring="neg_mean_absolute_error",
                    n_jobs=-1,
                )
                return scores.mean()
            return objective

        try:
            study = optuna.create_study(direction="maximize")
            per_backend_trials = max(n_trials // len(backends), 10)
            study.optimize(_make_objective(backend), n_trials=per_backend_trials, timeout=timeout // max(len(backends), 1))

            if study.best_value > best_score:
                best_score = study.best_value
                best_backend = backend
                best_params = dict(study.best_params)
        except Exception as e:
            logger.warning("Backend %s tuning failed: %s", backend, e)
            continue

    if best_params is None:
        return _default_xgb_regressor_params(), "xgboost"

    params = _finalize_params(best_params, best_backend, "regressor")
    logger.info("Best backend=%s, MAE=%.4f, params=%s", best_backend, best_score, params)
    return params, best_backend


def tune_xgb_classifier(
    X: np.ndarray,
    y: np.ndarray,
    n_trials: int = 30,
    timeout: int = 120,
) -> dict:
    params, backend = tune_classifier(X, y, backends=None, n_trials=n_trials, timeout=timeout)
    return params


def tune_xgb_regressor(
    X: np.ndarray,
    y: np.ndarray,
    n_trials: int = 30,
    timeout: int = 120,
) -> dict:
    params, backend = tune_regressor(X, y, backends=None, n_trials=n_trials, timeout=timeout)
    return params


def get_model_instance(params: dict, backend: str, task: str = "classifier"):
    if task == "classifier":
        if backend == "lightgbm" and _LIGHTGBM_AVAILABLE:
            from lightgbm import LGBMClassifier
            clean = {k: v for k, v in params.items() if k not in ("use_label_encoder", "eval_metric", "verbose")}
            return LGBMClassifier(**clean)
        elif backend == "catboost" and _CATBOOST_AVAILABLE:
            from catboost import CatBoostClassifier
            clean = {k: v for k, v in params.items() if k not in ("use_label_encoder", "eval_metric", "n_jobs", "subsample", "colsample_bytree")}
            clean["verbose"] = 0
            return CatBoostClassifier(**clean)
        else:
            from xgboost import XGBClassifier
            return XGBClassifier(**params)
    else:
        if backend == "lightgbm" and _LIGHTGBM_AVAILABLE:
            from lightgbm import LGBMRegressor
            clean = {k: v for k, v in params.items() if k not in ("use_label_encoder", "eval_metric", "verbose")}
            return LGBMRegressor(**clean)
        elif backend == "catboost" and _CATBOOST_AVAILABLE:
            from catboost import CatBoostRegressor
            clean = {k: v for k, v in params.items() if k not in ("use_label_encoder", "eval_metric", "n_jobs", "subsample", "colsample_bytree")}
            clean["verbose"] = 0
            return CatBoostRegressor(**clean)
        else:
            from xgboost import XGBRegressor
            return XGBRegressor(**params)


def _sample_classifier_params(trial, backend: str):
    if backend == "lightgbm":
        params = {
            "n_estimators": trial.suggest_int("n_estimators", 50, 300),
            "max_depth": trial.suggest_int("max_depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "subsample": trial.suggest_float("subsample", 0.6, 1.0),
            "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
            "min_child_samples": trial.suggest_int("min_child_samples", 5, 50),
            "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
            "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
            "random_state": 42,
            "verbose": -1,
        }
        from lightgbm import LGBMClassifier
        return params, LGBMClassifier
    elif backend == "catboost":
        params = {
            "iterations": trial.suggest_int("iterations", 50, 300),
            "depth": trial.suggest_int("depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "l2_leaf_reg": trial.suggest_float("l2_leaf_reg", 1e-8, 10.0, log=True),
            "random_seed": 42,
            "verbose": 0,
        }
        from catboost import CatBoostClassifier
        return params, CatBoostClassifier
    else:
        params = {
            "n_estimators": trial.suggest_int("n_estimators", 50, 300),
            "max_depth": trial.suggest_int("max_depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "subsample": trial.suggest_float("subsample", 0.6, 1.0),
            "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
            "min_child_weight": trial.suggest_int("min_child_weight", 1, 10),
            "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
            "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
            "random_state": 42,
            "use_label_encoder": False,
            "eval_metric": "logloss",
        }
        from xgboost import XGBClassifier
        return params, XGBClassifier


def _sample_regressor_params(trial, backend: str):
    if backend == "lightgbm":
        params = {
            "n_estimators": trial.suggest_int("n_estimators", 50, 300),
            "max_depth": trial.suggest_int("max_depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "subsample": trial.suggest_float("subsample", 0.6, 1.0),
            "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
            "min_child_samples": trial.suggest_int("min_child_samples", 5, 50),
            "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
            "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
            "random_state": 42,
            "verbose": -1,
        }
        from lightgbm import LGBMRegressor
        return params, LGBMRegressor
    elif backend == "catboost":
        params = {
            "iterations": trial.suggest_int("iterations", 50, 300),
            "depth": trial.suggest_int("depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "l2_leaf_reg": trial.suggest_float("l2_leaf_reg", 1e-8, 10.0, log=True),
            "random_seed": 42,
            "verbose": 0,
        }
        from catboost import CatBoostRegressor
        return params, CatBoostRegressor
    else:
        params = {
            "n_estimators": trial.suggest_int("n_estimators", 50, 300),
            "max_depth": trial.suggest_int("max_depth", 2, 8),
            "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
            "subsample": trial.suggest_float("subsample", 0.6, 1.0),
            "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
            "min_child_weight": trial.suggest_int("min_child_weight", 1, 10),
            "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
            "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
            "random_state": 42,
        }
        from xgboost import XGBRegressor
        return params, XGBRegressor


def _finalize_params(params: dict, backend: str, task: str) -> dict:
    result = dict(params)
    if backend == "xgboost":
        result["random_state"] = 42
        if task == "classifier":
            result["use_label_encoder"] = False
            result["eval_metric"] = "logloss"
    elif backend == "lightgbm":
        result["random_state"] = 42
        result["verbose"] = -1
    elif backend == "catboost":
        result["random_seed"] = 42
        result["verbose"] = 0
    result["_backend"] = backend
    return result


def _default_xgb_classifier_params() -> dict:
    return {
        "n_estimators": 100,
        "max_depth": 4,
        "learning_rate": 0.1,
        "subsample": 0.8,
        "colsample_bytree": 0.8,
        "random_state": 42,
        "use_label_encoder": False,
        "eval_metric": "logloss",
    }


def _default_xgb_regressor_params() -> dict:
    return {
        "n_estimators": 100,
        "max_depth": 4,
        "learning_rate": 0.1,
        "subsample": 0.8,
        "colsample_bytree": 0.8,
        "random_state": 42,
    }

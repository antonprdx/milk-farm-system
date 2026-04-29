import os
import shutil
from datetime import datetime


def save_with_version(path: str, data: object, max_backups: int = 3) -> None:
    import joblib

    if os.path.exists(path):
        ts = datetime.fromtimestamp(os.path.getmtime(path)).strftime("%Y%m%d_%H%M%S")
        backup_path = f"{path}.{ts}.bak"
        shutil.copy2(path, backup_path)

        backups = sorted(
            [f for f in os.listdir(os.path.dirname(path) or ".") if f.endswith(".bak")],
            reverse=True,
        )
        for old in backups[max_backups:]:
            os.remove(os.path.join(os.path.dirname(path) or ".", old))

    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    joblib.dump(data, path)

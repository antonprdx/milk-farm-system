#!/usr/bin/env bash
set -e

DB_USER="milkfarm"
DB_PASS="milkfarm"
DB_NAME="milkfarm"
DB_URL="postgres://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
ML_URL="http://localhost:8001"
ML_VENV="analytics-ml/.venv"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKIP_IMPORT=false
SKIP_TRAIN=false

for arg in "$@"; do
    case $arg in
        --skip-import) SKIP_IMPORT=true ;;
        --skip-train) SKIP_TRAIN=true ;;
    esac
done

echo "=== train-public-dataset.sh ==="
echo "Import FelBenitez dataset, augment, train all ML models"
echo ""

# Auto-detect: if milk_day_productions has data, skip import
MILK_COUNT=$(sudo -u postgres psql -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM milk_day_productions" 2>/dev/null || echo "0")
MILK_COUNT=$(echo "$MILK_COUNT" | tr -d '[:space:]')
if [ "$MILK_COUNT" -gt 100 ] 2>/dev/null; then
    echo "Found $MILK_COUNT rows in milk_day_productions (dataset already imported)."
    echo "  Use --skip-import to skip, or delete data to re-import."
    SKIP_IMPORT=true
fi

# Check if models already exist
MODEL_COUNT=$(ls "$SCRIPT_DIR/analytics-ml/models/"*.pkl 2>/dev/null | wc -l)
if [ "$MODEL_COUNT" -ge 6 ] 2>/dev/null; then
    echo "Found $MODEL_COUNT trained models."
    SKIP_TRAIN=true
fi

echo "[1/7] Starting PostgreSQL..."
sudo systemctl start postgresql

echo "[2/7] Ensuring database exists..."
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS;"
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME || \
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

echo "[3/7] Running migrations..."
(cd "$SCRIPT_DIR/backend" && DATABASE_URL="$DB_URL" cargo run --bin seed -- --cows 0 --years 0 2>/dev/null || true)

if [ "$SKIP_IMPORT" = true ]; then
    echo "[4/7] Skipping import (dataset already loaded)."
    echo "[5/7] Skipping augmentation."
else
    echo "  Truncating tables..."
    sudo -u postgres psql -d "$DB_NAME" -c "
        TRUNCATE TABLE vet_records, culling_events, heats, sync_log,
            grazing_data, bulk_tank_tests, ruminations, activities,
            feed_visits, feed_day_amounts, milk_visit_quality, robot_milk_data,
            milk_quality, milk_visits, milk_day_productions,
            dry_offs, pregnancies, inseminations, calves, calvings,
            bloodlines, transfers, feed_groups, feed_types, sires,
            animals, contacts, locations CASCADE;
    " 2>/dev/null || true

    echo "[4/7] Importing public dataset (FelBenitez, 210K rows)..."
    DATABASE_URL="$DB_URL" "$ML_VENV/bin/python" "$SCRIPT_DIR/analytics-ml/scripts/import_public_data.py"

    echo "[5/7] Augmenting with synthetic health/weather/activity data..."
    DATABASE_URL="$DB_URL" "$ML_VENV/bin/python" "$SCRIPT_DIR/analytics-ml/scripts/augment_real_data.py"
fi

echo "[6/7] Starting ML service for training..."
fuser -k 8001/tcp 2>/dev/null || true
if [ ! -d "$ML_VENV" ]; then
    echo "  Creating Python venv..."
    python3 -m venv "$ML_VENV"
    "$ML_VENV/bin/pip" install -q --upgrade pip
fi
if ! "$ML_VENV/bin/python" -c "import fastapi" 2>/dev/null; then
    echo "  Installing ML dependencies..."
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q \
        "fastapi" "uvicorn[standard]" "sqlalchemy[asyncio]" "asyncpg" \
        "pydantic" "pydantic-settings" "xgboost" "scikit-learn" \
        "pandas<3" "numpy" "joblib" "httpx" "apscheduler" 2>/dev/null || true
fi

ML_ENV="DATABASE_URL=postgresql+asyncpg://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
env $ML_ENV "$ML_VENV/bin/uvicorn" app.main:app --host 0.0.0.0 --port 8001 --app-dir "$SCRIPT_DIR/analytics-ml" &
ML_PID=$!

echo "  Waiting for ML service..."
for i in $(seq 1 30); do
    if curl -sf "$ML_URL/health" > /dev/null 2>&1; then
        break
    fi
    sleep 1
done

if ! curl -sf "$ML_URL/health" > /dev/null 2>&1; then
    echo "  ERROR: ML service failed to start"
    kill $ML_PID 2>/dev/null || true
    exit 1
fi
echo "  ML service ready."

if [ "$SKIP_TRAIN" = true ]; then
    echo "[7/7] Skipping training ($MODEL_COUNT models already exist)."
else
    echo "[7/7] Training all models..."
    ML_URL="$ML_URL" "$ML_VENV/bin/python" "$SCRIPT_DIR/analytics-ml/scripts/train_all.py"
fi

echo ""
echo "Stopping ML service..."
kill $ML_PID 2>/dev/null || true
wait $ML_PID 2>/dev/null || true

echo ""
echo "=== Done! ==="
echo "Models trained on FelBenitez public dataset (210K rows)."
echo "Run ./dev.sh to start the application with pre-trained models."

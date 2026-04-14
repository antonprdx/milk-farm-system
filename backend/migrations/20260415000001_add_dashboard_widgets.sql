ALTER TABLE user_preferences
    ADD COLUMN IF NOT EXISTS dashboard_widgets JSONB NOT NULL DEFAULT '["kpi","milk_trend","alerts","reproduction","feed","latest_milk","system_status","vet_followups","active_withdrawals","overdue_tasks"]'::jsonb;

PRAGMA foreign_keys=off;

CREATE TABLE IF NOT EXISTS worker_run (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  finished_at TIMESTAMP
);

ALTER TABLE product_history ADD COLUMN worker_run_id INTEGER REFERENCES worker_run(id);

PRAGMA foreign_keys=on;

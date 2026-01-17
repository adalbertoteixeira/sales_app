-- Create leads table
CREATE TABLE IF NOT EXISTS leads (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT NOT NULL,
email TEXT,
phone TEXT
) ;

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
id INTEGER PRIMARY KEY AUTOINCREMENT,
leads_id INTEGER NOT NULL,
sent_at TEXT,
reply_received TEXT,
reply_received_at TEXT,
ai_reply TEXT,
ai_reply_sent TEXT,
created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
status TEXT NOT NULL,
FOREIGN KEY (leads_id) REFERENCES leads (id)
) ;

-- Create outreach_log table
CREATE TABLE IF NOT EXISTS outreach_log (
id INTEGER PRIMARY KEY AUTOINCREMENT,
message_id INTEGER NOT NULL,
log_at TEXT NOT NULL,
step TEXT NOT NULL,
FOREIGN KEY (message_id) REFERENCES messages (id)
) ;

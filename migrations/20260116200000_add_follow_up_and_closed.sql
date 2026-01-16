-- Add follow_up_at column to messages table
ALTER TABLE messages ADD COLUMN follow_up_at TEXT;

-- Add closed_at column to messages table
ALTER TABLE messages ADD COLUMN closed_at TEXT;

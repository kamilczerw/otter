-- User-stored matching rules for transaction categorization.
-- `criteria` stores a JSON-serialized Vec<MatchCriterion>.
-- `position` is the evaluation order (first-rule-wins), assigned as
-- max(position)+1 on insert; unique so ordering is always well-defined.
CREATE TABLE matching_rules (
    id TEXT PRIMARY KEY,
    criteria TEXT NOT NULL,
    category TEXT NOT NULL,
    position INTEGER NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TRIGGER trg_matching_rules_updated_at
BEFORE UPDATE ON matching_rules
BEGIN
    UPDATE matching_rules SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;

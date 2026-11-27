ALTER TABLE prescriptions ADD COLUMN issued_snapshot TEXT CHECK (issued_snapshot IS NULL OR json_valid(issued_snapshot));

CREATE TRIGGER prescriptions_issued_snapshot_immutable
BEFORE UPDATE OF issued_snapshot ON prescriptions
WHEN OLD.issued_snapshot IS NOT NULL AND NEW.issued_snapshot IS NOT OLD.issued_snapshot
BEGIN
    SELECT RAISE(ABORT, 'issued snapshot is immutable');
END;

ALTER TABLE patients ADD COLUMN medical_record_no TEXT;
ALTER TABLE patients ADD COLUMN allergies TEXT;

ALTER TABLE prescriptions ADD COLUMN recetario_type TEXT NOT NULL DEFAULT 'ambulatorio' CHECK (recetario_type IN ('ambulatorio'));
ALTER TABLE prescriptions ADD COLUMN validity_days INTEGER CHECK (validity_days IN (1, 3));
ALTER TABLE prescriptions ADD COLUMN warning_signs TEXT;

ALTER TABLE prescription_items ADD COLUMN quantity_number INTEGER;

CREATE UNIQUE INDEX idx_patients_clinic_hcl ON patients (clinic_id, medical_record_no) WHERE medical_record_no IS NOT NULL;

DROP INDEX IF EXISTS idx_patients_clinic_hcl;

ALTER TABLE prescription_items DROP COLUMN quantity_number;

ALTER TABLE prescriptions DROP COLUMN warning_signs;
ALTER TABLE prescriptions DROP COLUMN validity_days;
ALTER TABLE prescriptions DROP COLUMN recetario_type;

ALTER TABLE patients DROP COLUMN allergies;
ALTER TABLE patients DROP COLUMN medical_record_no;

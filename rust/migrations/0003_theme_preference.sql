-- The student's colour theme: follow the device, or always light, or always dark.
ALTER TABLE student_profiles
    ADD COLUMN theme text NOT NULL DEFAULT 'system'
        CONSTRAINT student_profiles_theme_check CHECK (theme IN ('system', 'light', 'dark'));

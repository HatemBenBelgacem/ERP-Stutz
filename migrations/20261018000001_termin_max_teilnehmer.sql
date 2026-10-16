-- Maximale Anzahl Teilnehmende pro Termin
ALTER TABLE termin
    ADD COLUMN max_teilnehmer INTEGER NOT NULL DEFAULT 20 CHECK (max_teilnehmer > 0);

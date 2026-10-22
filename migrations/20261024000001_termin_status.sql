-- Bearbeitungsstand eines Termins; wird von Hand weitergesetzt.
CREATE TYPE termin_status AS ENUM ('neu', 'teamchefs_informiert', 'laufend', 'beendet');

ALTER TABLE termin
    ADD COLUMN status termin_status NOT NULL DEFAULT 'neu';

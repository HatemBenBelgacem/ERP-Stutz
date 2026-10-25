-- Mitarbeiter 1:1 Adresse: jede Adresse gehört höchstens einem Mitarbeiter.
-- Mitarbeiter N:1 Geschäftsbereich (Geschäftsbereich 1:N Mitarbeiter).
-- Wird die Adresse oder der Geschäftsbereich gelöscht, bleibt der Mitarbeiter ohne.
ALTER TABLE mitarbeiter
    ADD COLUMN adresse_id UUID UNIQUE REFERENCES adresse(id) ON DELETE SET NULL,
    ADD COLUMN geschaeftsbereich_id UUID REFERENCES geschaeftsbereich(id) ON DELETE SET NULL;

CREATE INDEX mitarbeiter_geschaeftsbereich_id_idx ON mitarbeiter(geschaeftsbereich_id);

-- Buchung N:1 Mitarbeiter: wer die Stunden geleistet hat. Bisherige Buchungen
-- bleiben ohne Mitarbeiter; Mitarbeiter mit Buchungen lassen sich nicht löschen.
ALTER TABLE buchung
    ADD COLUMN mitarbeiter_id UUID REFERENCES mitarbeiter(id) ON DELETE RESTRICT;

CREATE INDEX buchung_mitarbeiter_id_idx ON buchung(mitarbeiter_id);

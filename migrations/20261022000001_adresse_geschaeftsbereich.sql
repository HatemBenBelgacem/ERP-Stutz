-- Adressen: Durchführungsorte der Termine (Adresse 1:N Termin)
CREATE TABLE adresse (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bezeichnung  TEXT NOT NULL,
    strasse      TEXT NOT NULL DEFAULT '',
    plz          TEXT NOT NULL DEFAULT '',
    ort          TEXT NOT NULL DEFAULT ''
);

ALTER TABLE termin
    ADD COLUMN adresse_id UUID REFERENCES adresse(id) ON DELETE RESTRICT;

CREATE INDEX termin_adresse_id_idx ON termin(adresse_id);

-- Bisherige Orte werden zu Adressen; das Feld Ort entfällt
INSERT INTO adresse (bezeichnung)
    SELECT DISTINCT trim(ort) FROM termin WHERE trim(ort) <> '';
UPDATE termin t SET adresse_id = a.id
    FROM adresse a WHERE a.bezeichnung = trim(t.ort);
ALTER TABLE termin DROP COLUMN ort;

-- Geschäftsbereiche (Geschäftsbereich M:N Kurs)
CREATE TABLE geschaeftsbereich (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    beschreibung TEXT NOT NULL UNIQUE
);

CREATE TABLE kurs_geschaeftsbereich (
    kurs_id              UUID NOT NULL REFERENCES kurs(id) ON DELETE CASCADE,
    geschaeftsbereich_id UUID NOT NULL REFERENCES geschaeftsbereich(id) ON DELETE CASCADE,
    PRIMARY KEY (kurs_id, geschaeftsbereich_id)
);

CREATE INDEX kurs_geschaeftsbereich_geschaeftsbereich_id_idx ON kurs_geschaeftsbereich(geschaeftsbereich_id);

-- Wer Termine bzw. Kurse verwaltet, bekommt die neuen Bereiche dazu
UPDATE rolle SET rechte = rechte || '{adressen.sehen,adressen.anlegen,adressen.bearbeiten,adressen.loeschen}'
    WHERE NOT ist_admin AND 'termine.sehen' = ANY(rechte);
UPDATE rolle SET rechte = rechte || '{geschaeftsbereiche.sehen,geschaeftsbereiche.anlegen,geschaeftsbereiche.bearbeiten,geschaeftsbereiche.loeschen}'
    WHERE NOT ist_admin AND 'kurse.sehen' = ANY(rechte);

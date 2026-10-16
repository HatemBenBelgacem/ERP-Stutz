-- Kurse: Stammdaten der angebotenen Weiterbildungen
CREATE TABLE kurs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    titel         TEXT NOT NULL,
    beschreibung  TEXT NOT NULL DEFAULT '',
    dauer_stunden INTEGER NOT NULL DEFAULT 0 CHECK (dauer_stunden >= 0)
);

CREATE TABLE mitarbeiter (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vorname    TEXT NOT NULL,
    nachname   TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    abteilung  TEXT NOT NULL DEFAULT ''
);

-- Termin N:1 Kurs: jeder Termin gehört zu genau einem Kurs
CREATE TABLE termin (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kurs_id  UUID NOT NULL REFERENCES kurs(id) ON DELETE RESTRICT,
    beginn   TIMESTAMP NOT NULL,
    ende     TIMESTAMP NOT NULL,
    ort      TEXT NOT NULL DEFAULT '',
    CHECK (ende > beginn)
);

CREATE INDEX termin_kurs_id_idx ON termin(kurs_id);

-- Termin M:N Mitarbeiter: Teilnahmen
CREATE TABLE termin_mitarbeiter (
    termin_id      UUID NOT NULL REFERENCES termin(id) ON DELETE CASCADE,
    mitarbeiter_id UUID NOT NULL REFERENCES mitarbeiter(id) ON DELETE CASCADE,
    PRIMARY KEY (termin_id, mitarbeiter_id)
);

CREATE INDEX termin_mitarbeiter_mitarbeiter_id_idx ON termin_mitarbeiter(mitarbeiter_id);

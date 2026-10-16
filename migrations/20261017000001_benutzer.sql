-- Benutzerkonten für die Anmeldung an der App
CREATE TABLE benutzer (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT NOT NULL,
    email          TEXT NOT NULL UNIQUE,
    passwort_hash  TEXT NOT NULL,
    rolle          TEXT NOT NULL DEFAULT 'benutzer' CHECK (rolle IN ('admin', 'benutzer')),
    aktiv          BOOLEAN NOT NULL DEFAULT TRUE,
    erstellt_am    TIMESTAMP NOT NULL DEFAULT now()
);

-- Angemeldete Sitzungen; im Cookie steht nur das Token, hier sein SHA-256-Hash
CREATE TABLE sitzung (
    token_hash   TEXT PRIMARY KEY,
    benutzer_id  UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE,
    laeuft_ab    TIMESTAMP NOT NULL
);

CREATE INDEX sitzung_benutzer_id_idx ON sitzung(benutzer_id);

-- Rollen bündeln Rechte im Format "<bereich>.<aktion>", z. B. "termine.bearbeiten".
-- Eine Admin-Rolle (ist_admin) hat immer alle Rechte.
CREATE TABLE rolle (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT NOT NULL UNIQUE,
    beschreibung TEXT NOT NULL DEFAULT '',
    ist_admin    BOOLEAN NOT NULL DEFAULT FALSE,
    rechte       TEXT[] NOT NULL DEFAULT '{}',
    erstellt_am  TIMESTAMP NOT NULL DEFAULT now()
);

INSERT INTO rolle (name, beschreibung, ist_admin)
VALUES ('Administrator', 'Alle Rechte, inklusive Einstellungen', TRUE);

INSERT INTO rolle (name, beschreibung, rechte)
VALUES ('Sachbearbeitung', 'Termine, Kurse und Mitarbeiter verwalten', '{
    uebersicht.sehen,
    termine.sehen, termine.anlegen, termine.bearbeiten, termine.loeschen,
    kurse.sehen, kurse.anlegen, kurse.bearbeiten, kurse.loeschen,
    mitarbeiter.sehen, mitarbeiter.anlegen, mitarbeiter.bearbeiten, mitarbeiter.loeschen
}');

-- Benutzer ohne Rolle haben keinen Zugriff. Rollen mit Benutzern lassen sich nicht löschen.
ALTER TABLE benutzer
    ADD COLUMN rolle_id UUID REFERENCES rolle(id) ON DELETE RESTRICT,
    ADD COLUMN microsoft_oid TEXT UNIQUE;

CREATE INDEX benutzer_rolle_id_idx ON benutzer(rolle_id);

-- Bisherige Admins → Administrator, alle mit freigegebenen Bereichen → Sachbearbeitung
UPDATE benutzer SET rolle_id = (SELECT id FROM rolle WHERE ist_admin) WHERE rolle = 'admin';
UPDATE benutzer SET rolle_id = (SELECT id FROM rolle WHERE name = 'Sachbearbeitung')
    WHERE rolle <> 'admin' AND cardinality(berechtigungen) > 0;

ALTER TABLE benutzer DROP COLUMN rolle, DROP COLUMN berechtigungen;

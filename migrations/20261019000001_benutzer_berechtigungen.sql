-- Bereiche, die ein Benutzer sehen darf (Admins sehen immer alles).
-- Neue Konten starten ohne Bereiche, bis ein Admin sie freigibt.
ALTER TABLE benutzer
    ADD COLUMN berechtigungen TEXT[] NOT NULL DEFAULT '{}';

-- Bestehende Konten behalten ihren bisherigen Zugriff auf alles
UPDATE benutzer SET berechtigungen = '{uebersicht,termine,kurse,mitarbeiter}';

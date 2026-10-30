//! Module der App: Menüeinträge in der Seitenleiste und Kacheln auf der
//! Startansicht nutzen dieselbe Liste.

use crate::models::Bereich;

/// Ein Modul: Menüeintrag und Kachel zugleich.
#[derive(Clone, Copy)]
pub struct Ziel {
    pub text: &'static str,
    pub href: &'static str,
    pub icon: icondata::Icon,
    pub bereiche: &'static [Bereich],
}

impl Ziel {
    /// Sichtbar, sobald der Benutzer mindestens einen seiner Bereiche sehen darf.
    pub fn sichtbar(&self, darf: impl Fn(Bereich) -> bool) -> bool {
        self.bereiche.iter().any(|b| darf(*b))
    }
}

/// Navigation in Gruppen; Gruppen mit Titel lassen sich einklappen,
/// Gruppen ohne Titel stehen für sich.
pub const NAVIGATION: [(Option<(&str, icondata::Icon)>, &[Ziel]); 2] = [
    (
        Some(("Personal", icondata::LuBriefcase)),
        &[
            // Termine, Kurse und Anmeldung als Reiter
            Ziel {
                text: "Weiterbildung",
                href: "/personal/weiterbildung",
                icon: icondata::LuGraduationCap,
                bereiche: &Bereich::WEITERBILDUNG,
            },
            Ziel {
                text: "Mitarbeiter",
                href: "/personal/mitarbeiter",
                icon: icondata::LuUsers,
                bereiche: &[Bereich::Mitarbeiter],
            },
        ],
    ),
    (
        None,
        &[
            Ziel {
                text: "Adressen",
                href: "/adressen",
                icon: icondata::LuMapPin,
                bereiche: &[Bereich::Adressen],
            },
            Ziel {
                text: "Einkauf",
                href: "/einkauf",
                icon: icondata::LuShoppingCart,
                bereiche: &[Bereich::Einkauf],
            },
            Ziel {
                text: "Lager",
                href: "/lager",
                icon: icondata::LuWarehouse,
                bereiche: &[Bereich::Lager],
            },
            Ziel {
                text: "Artikel",
                href: "/lager/artikel",
                icon: icondata::LuPackage,
                bereiche: &[Bereich::Lager],
            },
            // Baumansicht Bauobjekte → Projekte
            Ziel {
                text: "Projekte",
                href: "/projekte",
                icon: icondata::LuConstruction,
                bereiche: &Bereich::PROJEKTE,
            },
            Ziel {
                text: "Stunden erfassen",
                href: "/stunden-erfassen",
                icon: icondata::LuClockPlus,
                bereiche: &[Bereich::Stunden],
            },
            Ziel {
                text: "Buchungen",
                href: "/buchungen",
                icon: icondata::LuReceipt,
                bereiche: &[Bereich::Buchungen],
            },
            Ziel {
                text: "Werkhof",
                href: "/werkhof",
                icon: icondata::LuTruck,
                bereiche: &[Bereich::Werkhof],
            },
            Ziel {
                text: "Finanzen",
                href: "/finanzen",
                icon: icondata::LuWallet,
                bereiche: &[Bereich::Finanzen],
            },
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Alle Module in der Reihenfolge der Navigation
    fn module() -> Vec<Ziel> {
        NAVIGATION
            .iter()
            .flat_map(|(_, eintraege)| eintraege.iter().copied())
            .collect()
    }

    fn modul(text: &str) -> Ziel {
        module()
            .into_iter()
            .find(|z| z.text == text)
            .expect("Modul fehlt in der Navigation")
    }

    #[test]
    fn elf_module() {
        assert_eq!(module().len(), 11);
    }

    #[test]
    fn weiterbildung_schon_mit_einem_reiter_sichtbar() {
        let weiterbildung = modul("Weiterbildung");
        assert!(weiterbildung.sichtbar(|b| b == Bereich::Kurse));
        assert!(!weiterbildung.sichtbar(|b| b == Bereich::Buchungen));
    }

    #[test]
    fn projekte_schon_mit_bauobjekten_sichtbar() {
        let projekte = modul("Projekte");
        assert!(projekte.sichtbar(|b| b == Bereich::Bauobjekte));
        assert!(!projekte.sichtbar(|b| b == Bereich::Mitarbeiter));
    }
}

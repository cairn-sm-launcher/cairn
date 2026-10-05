//! Imprime le bareme de rang, pour le juger sur pieces.
//!
//! `cargo run --example apercu_rang` depuis `crates/cairn-core`.
//!
//! Les regles sont testees, mais un test dit qu'un calcul est conforme a
//! ce qu'on a decide, pas que ce qu'on a decide est agreable a jouer.
//! Ce tableau la sert a ca : regarder les chiffres avant de les figer.

use cairn_core::rang::{self, Affrontement, Joueur, Palier, Rang};

fn joueur(palier: u8, consecutifs: u32, vient_de_monter: bool) -> Joueur {
    Joueur {
        rang: Rang::nouveau(Palier::depuis_indice(palier), 50),
        consecutifs,
        deja_monte: true,
        vient_de_monter,
    }
}

fn duel(gagne: bool, mien: u8, adverse: u8) -> Affrontement {
    Affrontement {
        gagne,
        mon_palier: mien,
        palier_adverse: adverse,
    }
}

const NOMS: [&str; 5] = ["Bronze", "Argent", "Or", "Platine", "Diamant"];

fn main() {
    println!(
        "Bareme : palier {} points, gain de base {}, perte de base {}, plafond {}.",
        rang::POINTS_PAR_PALIER,
        rang::GAIN_BASE,
        rang::PERTE_BASE,
        rang::PLAFOND_PAR_MATCH
    );
    println!();

    println!("VICTOIRE, selon l'adversaire (lignes : mon palier)");
    println!(
        "{:>9} | {:>8} {:>8} {:>8} {:>8} {:>8}",
        "", NOMS[0], NOMS[1], NOMS[2], NOMS[3], NOMS[4]
    );
    for mien in 0..=4u8 {
        print!("{:>9} |", NOMS[mien as usize]);
        for adverse in 0..=4u8 {
            let r = rang::calculer(&joueur(mien, 1, false), &duel(true, mien, adverse));
            print!(" {:>8}", format!("+{}", r.delta));
        }
        println!();
    }
    println!();

    println!("DEFAITE, selon l'adversaire");
    println!(
        "{:>9} | {:>8} {:>8} {:>8} {:>8} {:>8}",
        "", NOMS[0], NOMS[1], NOMS[2], NOMS[3], NOMS[4]
    );
    for mien in 0..=4u8 {
        print!("{:>9} |", NOMS[mien as usize]);
        for adverse in 0..=4u8 {
            let r = rang::calculer(&joueur(mien, 1, false), &duel(false, mien, adverse));
            print!(" {:>8}", r.delta);
        }
        println!();
    }
    println!();

    println!("L'EFFET DES SERIES, a niveau egal");
    println!("{:>14} | {:>9} {:>9}", "consecutifs", "victoire", "defaite");
    for n in 1..=6u32 {
        let v = rang::calculer(&joueur(2, n, false), &duel(true, 2, 2));
        let d = rang::calculer(&joueur(2, n, false), &duel(false, 2, 2));
        println!("{n:>14} | {:>9} {:>9}", format!("+{}", v.delta), d.delta);
    }
    println!();

    println!("LES CAS EXTREMES");
    let cas: [(&str, Joueur, Affrontement); 5] = [
        (
            "meilleur match possible",
            joueur(0, 6, false),
            duel(true, 0, 4),
        ),
        ("pire match possible", joueur(4, 6, true), duel(false, 4, 0)),
        ("Diamant bat Bronze", joueur(4, 1, false), duel(true, 4, 0)),
        (
            "Bronze perd contre Diamant",
            joueur(0, 1, false),
            duel(false, 0, 4),
        ),
        (
            "defaite juste apres une montee",
            joueur(2, 1, true),
            duel(false, 2, 3),
        ),
    ];
    for (nom, j, a) in cas {
        let r = rang::calculer(&j, &a);
        let detail: Vec<String> = r
            .effets
            .iter()
            .map(|e| format!("{:?} {:+}%", e.regle, e.pourcentage))
            .collect();
        println!("{:>31} : {:>4}   {}", nom, r.delta, detail.join(", "));
    }
    println!();

    println!("COMBIEN DE MATCHS POUR MONTER D'UN PALIER, a niveau egal");
    for (nom, gagne) in [("victoires d'affilee", true), ("defaites d'affilee", false)] {
        let mut j = joueur(2, 0, false);
        j.rang = Rang::nouveau(Palier::Or, if gagne { 0 } else { 99 });
        let depart = j.rang.palier;
        let mut n = 0;
        while j.rang.palier == depart && n < 40 {
            n += 1;
            j.consecutifs = n;
            let r = rang::calculer(&j, &duel(gagne, 2, 2));
            j.rang = r.rang_apres;
        }
        println!("{:>22} : {} matchs", nom, n);
    }
}

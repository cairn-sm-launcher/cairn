//! L'outil de publication du fil d'actualites.
//!
//! Il fait deux choses, et rien d'autre : fabriquer un trousseau, et
//! signer un fichier.
//!
//! ```text
//! cargo run --example actualites -- cle
//! cargo run --example actualites -- signer <cle-privee.txt> <actualites.json>
//! ```
//!
//! **La moitie privee ne doit jamais sortir de la machine qui publie.**
//! Pas dans le depot, pas dans un ticket, pas dans une conversation. Si
//! elle fuit, n'importe qui peut fabriquer un fil que tous les Cairn
//! installes croiront : c'est le seul secret du dispositif, et c'est
//! aussi tout ce qu'il demande de garder.
//!
//! Si elle est perdue, rien n'est casse : on refait un trousseau, on
//! remplace `CLE_ACTUALITES` et on publie une version. Les Cairn deja
//! installes cessent simplement de recevoir du neuf jusqu'a la mise a
//! jour, et gardent ce qu'ils ont en cache.

use ed25519_dalek::{Signer, SigningKey};
use std::io::Write;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("cle") => fabriquer_un_trousseau(),
        Some("signer") if arguments.len() == 3 => signer(&arguments[1], &arguments[2]),
        _ => {
            eprintln!("Usage :");
            eprintln!("  cargo run --example actualites -- cle");
            eprintln!("  cargo run --example actualites -- signer <cle-privee.txt> <fichier>");
            std::process::exit(2);
        }
    }
}

fn fabriquer_un_trousseau() {
    let mut graine = [0u8; 32];
    getrandom::fill(&mut graine).expect("pas de source aleatoire sur ce systeme");
    let privee = SigningKey::from_bytes(&graine);
    let publique = privee.verifying_key();

    let nom = "cairn-actualites-privee.txt";
    let mut fichier = std::fs::File::create(nom).expect("ecriture de la cle privee");
    writeln!(fichier, "{}", en_hex(&privee.to_bytes())).expect("ecriture de la cle privee");

    println!("Cle publique, a mettre dans cairn_core::CLE_ACTUALITES :");
    println!();
    println!("{}", en_hex(publique.as_bytes()));
    println!();
    println!("Cle privee ecrite dans {nom}.");
    println!("Garde la hors du depot. Elle ne doit jamais etre partagee.");
}

fn signer(chemin_cle: &str, chemin_fichier: &str) {
    let texte = std::fs::read_to_string(chemin_cle).expect("lecture de la cle privee");
    let octets = depuis_hex(texte.trim()).expect("la cle privee doit faire 64 caracteres hex");
    let graine: [u8; 32] = octets
        .try_into()
        .expect("la cle privee doit faire 32 octets");
    let privee = SigningKey::from_bytes(&graine);

    // On signe le fichier **tel qu'il est sur le disque**. Pas de
    // relecture JSON, pas de reserialisation : la signature porte sur les
    // octets qui seront servis, et c'est ce qui la rend verifiable sans
    // avoir a se mettre d'accord sur une normalisation.
    let contenu = std::fs::read(chemin_fichier).expect("lecture du fichier a signer");
    let signature = privee.sign(&contenu);

    let sortie = format!("{chemin_fichier}.sig");
    std::fs::write(&sortie, en_hex(&signature.to_bytes())).expect("ecriture de la signature");
    println!("Signature ecrite dans {sortie}.");
    println!("Publie les deux fichiers cote a cote.");
}

fn en_hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

fn depuis_hex(valeur: &str) -> Option<Vec<u8>> {
    if valeur.len() % 2 != 0 {
        return None;
    }
    (0..valeur.len() / 2)
        .map(|i| u8::from_str_radix(&valeur[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

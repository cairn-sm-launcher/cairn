//! Garder lisible ce que le jeu a colore.
//!
//! Les noms de serveurs portent les couleurs que leur proprietaire a
//! choisies, et il les a choisies en regardant le jeu, pas Cairn. Le
//! menu de ManiaPlanet pose ces noms sur des fonds clairs et varies.
//! Cairn les pose sur un panneau sombre et fixe. Un `$000`, un `$111`
//! ou un bleu nuit y deviennent tout simplement invisibles.
//!
//! **On ne refuse pas la couleur, on la remonte.** Effacer les couleurs
//! serait plus simple et ce serait une perte : un serveur se reconnait a
//! son nom colore, c'est son identite. Remplacer la couleur par du blanc
//! serait pire encore, tout se ressemblerait. On garde donc la teinte et
//! la saturation, et on monte la clarte juste assez pour atteindre un
//! contraste lisible. Un rouge reste rouge, un bleu nuit devient un bleu
//! plus clair, et un noir devient un gris.
//!
//! **Ce que ce module ne fait pas :** il ne baisse jamais la clarte. Sur
//! un fond sombre, un texte trop clair n'a jamais pose de probleme.

/// Le fond sur lequel on calcule.
///
/// `#13161B`, celui des panneaux et des tuiles, et non le `#0A0C0F` de
/// la page. C'est le plus clair des deux, donc le plus exigeant pour une
/// couleur sombre : une couleur qui passe ici passe partout dans Cairn.
pub const FOND_REFERENCE: [u8; 3] = [0x13, 0x16, 0x1B];

/// Le contraste vise, en rapport de luminance.
///
/// 4,5 est la barre que WCAG pose pour du texte courant, et un nom de
/// serveur en est : on le lit, on le compare, on clique dessus. Une
/// barre plus basse, 3 par exemple, garderait des teintes plus franches
/// et laisserait passer des bleus qui se devinent plus qu'ils ne se
/// lisent. Le chiffre est ici, en une ligne, si l'usage dit le
/// contraire.
pub const CONTRASTE_CIBLE: f64 = 4.5;

/// La luminance relative d'une couleur, au sens de WCAG.
pub fn luminance(couleur: [u8; 3]) -> f64 {
    fn lineaire(composante: u8) -> f64 {
        let c = composante as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * lineaire(couleur[0]) + 0.7152 * lineaire(couleur[1]) + 0.0722 * lineaire(couleur[2])
}

/// Le rapport de contraste entre deux couleurs, de 1 a 21.
pub fn contraste(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (haut, bas) = if la > lb { (la, lb) } else { (lb, la) };
    (haut + 0.05) / (bas + 0.05)
}

/// Remonte une couleur jusqu'au contraste vise, sans toucher a sa teinte.
///
/// Rend la couleur telle quelle si elle passe deja. Sinon elle est
/// eclaircie par recherche dichotomique sur la clarte, ce qui donne la
/// couleur la **plus proche de l'originale** qui soit lisible : on
/// n'eclaircit pas d'un cran arbitraire, on eclaircit du minimum
/// necessaire.
pub fn ajuster(couleur: [u8; 3], fond: [u8; 3], cible: f64) -> [u8; 3] {
    if contraste(couleur, fond) >= cible {
        return couleur;
    }
    let (teinte, saturation, clarte) = vers_tsl(couleur);

    // Si meme le blanc ne suffit pas, c'est que le fond est clair : ce
    // module ne sait pas assombrir, et il vaut mieux rendre le blanc que
    // de boucler pour rien.
    if contraste([255, 255, 255], fond) < cible {
        return [255, 255, 255];
    }

    let (mut bas, mut haut) = (clarte, 1.0);
    // Vingt tours suffisent largement : le pas final vaut moins d'un
    // millioniemme de clarte, bien en dessous de ce qu'un octet distingue.
    for _ in 0..20 {
        let milieu = (bas + haut) / 2.0;
        if contraste(depuis_tsl(teinte, saturation, milieu), fond) >= cible {
            haut = milieu;
        } else {
            bas = milieu;
        }
    }
    let resultat = depuis_tsl(teinte, saturation, haut);
    // L'arrondi en octets peut faire retomber d'un cheveu sous la cible.
    if contraste(resultat, fond) >= cible {
        resultat
    } else {
        depuis_tsl(teinte, saturation, (haut + 0.004).min(1.0))
    }
}

/// La meme chose, avec le fond et la cible de Cairn.
pub fn lisible(couleur: [u8; 3]) -> [u8; 3] {
    ajuster(couleur, FOND_REFERENCE, CONTRASTE_CIBLE)
}

/// `#RRGGBB`, la forme qu'attend un attribut `style`.
pub fn en_hexadecimal(couleur: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", couleur[0], couleur[1], couleur[2])
}

// ---------------------------------------------------------------------
//  Teinte, saturation, clarte
// ---------------------------------------------------------------------
//
//  On passe par la parce que c'est le seul espace simple ou « eclaircir
//  sans changer la couleur » est une seule coordonnee. Melanger vers le
//  blanc marcherait aussi mais delaverait la teinte au passage, et c'est
//  exactement ce qu'on veut eviter.

fn vers_tsl(couleur: [u8; 3]) -> (f64, f64, f64) {
    let [r, v, b] = couleur.map(|c| c as f64 / 255.0);
    let max = r.max(v).max(b);
    let min = r.min(v).min(b);
    let clarte = (max + min) / 2.0;
    let delta = max - min;
    if delta.abs() < f64::EPSILON {
        return (0.0, 0.0, clarte);
    }
    let saturation = delta / (1.0 - (2.0 * clarte - 1.0).abs());
    let teinte = if max == r {
        60.0 * (((v - b) / delta) % 6.0)
    } else if max == v {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - v) / delta + 4.0)
    };
    ((teinte + 360.0) % 360.0, saturation, clarte)
}

fn depuis_tsl(teinte: f64, saturation: f64, clarte: f64) -> [u8; 3] {
    let c = (1.0 - (2.0 * clarte - 1.0).abs()) * saturation;
    let x = c * (1.0 - ((teinte / 60.0) % 2.0 - 1.0).abs());
    let m = clarte - c / 2.0;
    let (r, v, b) = match teinte {
        t if t < 60.0 => (c, x, 0.0),
        t if t < 120.0 => (x, c, 0.0),
        t if t < 180.0 => (0.0, c, x),
        t if t < 240.0 => (0.0, x, c),
        t if t < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r, v, b].map(|composante| (((composante + m) * 255.0).round()).clamp(0.0, 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_reperes_de_luminance_sont_justes() {
        assert!((luminance([0, 0, 0])).abs() < 1e-9);
        assert!((luminance([255, 255, 255]) - 1.0).abs() < 1e-9);
        // Le blanc sur le noir, c'est le contraste maximal de l'echelle.
        assert!((contraste([255, 255, 255], [0, 0, 0]) - 21.0).abs() < 1e-6);
        // Le contraste ne depend pas de l'ordre.
        let a = contraste([0x20, 0xE3, 0xB2], FOND_REFERENCE);
        let b = contraste(FOND_REFERENCE, [0x20, 0xE3, 0xB2]);
        assert!((a - b).abs() < 1e-9);
    }

    #[test]
    fn une_couleur_deja_lisible_ne_bouge_pas() {
        for couleur in [[0xFF, 0xFF, 0xFF], [0x20, 0xE3, 0xB2], [0xFF, 0xC8, 0x57]] {
            assert_eq!(lisible(couleur), couleur);
        }
    }

    #[test]
    fn le_noir_devient_un_gris_lisible() {
        let corrige = lisible([0, 0, 0]);
        assert!(contraste(corrige, FOND_REFERENCE) >= CONTRASTE_CIBLE);
        // Un gris : les trois composantes restent egales.
        assert_eq!(corrige[0], corrige[1]);
        assert_eq!(corrige[1], corrige[2]);
        // Et il ne part pas jusqu'au blanc, on monte du minimum necessaire.
        assert!(corrige[0] < 255);
    }

    #[test]
    fn un_bleu_nuit_reste_bleu() {
        let corrige = lisible([0x00, 0x00, 0x55]);
        assert!(contraste(corrige, FOND_REFERENCE) >= CONTRASTE_CIBLE);
        // Le bleu domine toujours largement.
        assert!(corrige[2] > corrige[0] + 40);
        assert!(corrige[2] > corrige[1] + 40);
    }

    #[test]
    fn toutes_les_couleurs_du_jeu_passent_la_barre() {
        // Les 4096 couleurs que `$RGB` peut ecrire, quartet duplique.
        for r in 0..16u8 {
            for v in 0..16u8 {
                for b in 0..16u8 {
                    let couleur = [r * 17, v * 17, b * 17];
                    let corrige = lisible(couleur);
                    assert!(
                        contraste(corrige, FOND_REFERENCE) >= CONTRASTE_CIBLE - 1e-9,
                        "{couleur:?} corrige en {corrige:?} reste sous la barre"
                    );
                }
            }
        }
    }

    #[test]
    fn la_correction_ne_fait_jamais_que_monter() {
        for r in 0..16u8 {
            for v in 0..16u8 {
                for b in 0..16u8 {
                    let couleur = [r * 17, v * 17, b * 17];
                    let corrige = lisible(couleur);
                    assert!(
                        luminance(corrige) >= luminance(couleur) - 1e-9,
                        "{couleur:?} a ete assombri en {corrige:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn la_teinte_est_conservee() {
        for couleur in [[0x00, 0x00, 0x55], [0x22, 0x00, 0x00], [0x00, 0x33, 0x11]] {
            let (teinte, _, _) = vers_tsl(couleur);
            let (teinte_corrigee, _, _) = vers_tsl(lisible(couleur));
            let ecart = (teinte - teinte_corrigee).abs().min(
                // La teinte est un angle, 359 et 1 sont voisins.
                360.0 - (teinte - teinte_corrigee).abs(),
            );
            assert!(
                ecart < 2.0,
                "{couleur:?} a change de teinte de {ecart} degres"
            );
        }
    }

    #[test]
    fn l_aller_retour_tsl_ne_deforme_pas() {
        for couleur in [
            [0x12, 0x34, 0x56],
            [0xFF, 0x00, 0x00],
            [0x00, 0xFF, 0x00],
            [0x00, 0x00, 0xFF],
            [0x80, 0x80, 0x80],
        ] {
            let (t, s, c) = vers_tsl(couleur);
            let retour = depuis_tsl(t, s, c);
            for rang in 0..3 {
                assert!(
                    (retour[rang] as i16 - couleur[rang] as i16).abs() <= 1,
                    "{couleur:?} revient en {retour:?}"
                );
            }
        }
    }

    #[test]
    fn la_forme_hexadecimale_est_celle_du_css() {
        assert_eq!(en_hexadecimal([0x20, 0xE3, 0xB2]), "#20E3B2");
        assert_eq!(en_hexadecimal([0, 0, 0]), "#000000");
    }
}

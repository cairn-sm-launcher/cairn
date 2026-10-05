//! Les raccourcis clavier, lies a la **position** de la touche.
//!
//! La proposition d'origine etait de changer les touches selon la
//! disposition du clavier, A et E en AZERTY, Q et E en QWERTY. On fait
//! mieux et plus simple : **on ne detecte pas la disposition du tout.**
//!
//! Un evenement clavier porte deux informations, le caractere produit et
//! la position physique de la touche. La touche qui porte « A » sur un
//! clavier francais et « Q » sur un clavier anglais est la meme touche,
//! et elle s'appelle `KeyQ` dans les deux cas. On lie donc l'action a la
//! position, une seule fois, et ca vaut pour l'AZERTY, le QWERTY, le
//! QWERTZ, le Dvorak et le Bepo, sans table a maintenir.
//!
//! L'etiquette affichee, elle, est demandee au systeme par l'interface,
//! ce qui montre « A » chez un francophone et « Q » chez un anglophone
//! sans que personne n'ait rien regle. Le repli, quand le systeme ne
//! repond pas, est [`etiquette_de_repli`].
//!
//! Ce module ne connait donc que des codes de position. Il n'y a pas une
//! ligne ici qui sache ce qu'est un clavier francais.

/// La touche qui ramene vers l'onglet de gauche.
///
/// `KeyQ` : etiquetee A en AZERTY, Q en QWERTY.
pub const PRECEDENT_DEFAUT: &str = "KeyQ";

/// La touche qui pousse vers l'onglet de droite. Etiquetee E partout ou
/// la rangee du haut suit l'ordre habituel.
pub const SUIVANT_DEFAUT: &str = "KeyE";

/// Les codes que Cairn refuse comme raccourci.
///
/// Ils ne sont pas interdits pour le principe, chacun a sa raison. Les
/// modificateurs ne produisent pas d'evenement utilisable seuls.
/// `Tab` deplace le focus, c'est la navigation au clavier de toute
/// l'interface. `Enter` et `Space` activent l'element focalise.
/// `Escape` ferme les surimpressions. Les reaffecter reviendrait a
/// casser quelque chose qui marche pour gagner un raccourci.
const INTERDITS: [&str; 12] = [
    "Tab",
    "Enter",
    "NumpadEnter",
    "Escape",
    "Space",
    "ShiftLeft",
    "ShiftRight",
    "ControlLeft",
    "ControlRight",
    "AltLeft",
    "AltRight",
    "MetaLeft",
];

/// Vrai quand ce code de position peut servir de raccourci.
///
/// La liste des formes acceptees est volontairement etroite : ce qui
/// entre ici vient d'un evenement clavier, donc du monde exterieur, et
/// finira compare a un `event.code`. Une forme qu'on ne reconnait pas
/// n'a aucune chance d'etre un vrai code de touche.
pub fn code_valide(code: &str) -> bool {
    if INTERDITS.contains(&code) {
        return false;
    }
    fn suffixe_unique(code: &str, prefixe: &str, admis: fn(char) -> bool) -> bool {
        match code.strip_prefix(prefixe) {
            Some(reste) => {
                let mut c = reste.chars();
                matches!((c.next(), c.next()), (Some(x), None) if admis(x))
            }
            None => false,
        }
    }

    suffixe_unique(code, "Key", |c| c.is_ascii_uppercase())
        || suffixe_unique(code, "Digit", |c| c.is_ascii_digit())
        || suffixe_unique(code, "Numpad", |c| c.is_ascii_digit())
        || [
            "Minus",
            "Equal",
            "BracketLeft",
            "BracketRight",
            "Backslash",
            "Semicolon",
            "Quote",
            "Backquote",
            "Comma",
            "Period",
            "Slash",
            "IntlBackslash",
            "ArrowUp",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "Home",
            "End",
            "PageUp",
            "PageDown",
            "Insert",
            "F1",
            "F2",
            "F3",
            "F4",
            "F5",
            "F6",
            "F7",
            "F8",
            "F9",
            "F10",
            "F11",
            "F12",
        ]
        .contains(&code)
}

/// Ramene un couple de raccourcis dans le domaine du possible.
///
/// Un code invalide reprend sa valeur par defaut. Et les deux touches ne
/// peuvent pas etre la meme : une touche qui irait a la fois a gauche et
/// a droite ne menerait nulle part, alors on rend la paire par defaut
/// plutot que d'en choisir une au hasard.
pub fn assainir(precedent: &str, suivant: &str) -> (String, String) {
    let p = if code_valide(precedent) {
        precedent
    } else {
        PRECEDENT_DEFAUT
    };
    let s = if code_valide(suivant) {
        suivant
    } else {
        SUIVANT_DEFAUT
    };
    if p == s {
        return (PRECEDENT_DEFAUT.to_string(), SUIVANT_DEFAUT.to_string());
    }
    (p.to_string(), s.to_string())
}

/// L'etiquette a afficher quand le systeme ne sait pas la donner.
///
/// Elle suppose un clavier QWERTY, ce qui est faux chez Flow. C'est
/// assume : ce n'est qu'un repli, l'interface demande d'abord au systeme
/// la vraie etiquette de la position, et tous les navigateurs recents
/// savent repondre. Mieux vaut « Q » sur une pastille que rien du tout.
pub fn etiquette_de_repli(code: &str) -> String {
    if let Some(lettre) = code.strip_prefix("Key") {
        return lettre.to_string();
    }
    if let Some(chiffre) = code.strip_prefix("Digit") {
        return chiffre.to_string();
    }
    if let Some(chiffre) = code.strip_prefix("Numpad") {
        return format!("№{chiffre}");
    }
    match code {
        "ArrowUp" => "↑".to_string(),
        "ArrowDown" => "↓".to_string(),
        "ArrowLeft" => "←".to_string(),
        "ArrowRight" => "→".to_string(),
        "Comma" => ",".to_string(),
        "Period" => ".".to_string(),
        "Slash" => "/".to_string(),
        "Semicolon" => ";".to_string(),
        "Quote" => "'".to_string(),
        "Backquote" => "`".to_string(),
        "Minus" => "-".to_string(),
        "Equal" => "=".to_string(),
        "BracketLeft" => "[".to_string(),
        "BracketRight" => "]".to_string(),
        "Backslash" => "\\".to_string(),
        "IntlBackslash" => "<".to_string(),
        autre => autre.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_defauts_sont_des_positions_et_pas_des_lettres() {
        // Le point de tout le module : KeyQ est la touche etiquetee A en
        // AZERTY et Q en QWERTY. On ne stocke jamais « A ».
        assert_eq!(PRECEDENT_DEFAUT, "KeyQ");
        assert_eq!(SUIVANT_DEFAUT, "KeyE");
        assert!(code_valide(PRECEDENT_DEFAUT));
        assert!(code_valide(SUIVANT_DEFAUT));
    }

    #[test]
    fn les_formes_de_codes_reconnues() {
        for bon in [
            "KeyA",
            "KeyZ",
            "Digit0",
            "Digit9",
            "Numpad5",
            "ArrowLeft",
            "F5",
            "Comma",
            "Slash",
            "IntlBackslash",
        ] {
            assert!(code_valide(bon), "{bon} devrait etre accepte");
        }
    }

    #[test]
    fn ce_qui_n_est_pas_un_code_de_touche_est_refuse() {
        for mauvais in [
            "",
            "A",
            "key a",
            "KeyAB",
            "Keya",
            "Key",
            "Digit10",
            "DigitA",
            "F13",
            "n'importe quoi",
        ] {
            assert!(!code_valide(mauvais), "{mauvais:?} devrait etre refuse");
        }
    }

    #[test]
    fn les_touches_qui_font_deja_quelque_chose_sont_refusees() {
        // Tab deplace le focus, Enter et Space activent, Escape ferme.
        // Les reaffecter casserait la navigation au clavier.
        for reservee in [
            "Tab",
            "Enter",
            "Escape",
            "Space",
            "ShiftLeft",
            "ControlLeft",
        ] {
            assert!(!code_valide(reservee), "{reservee} devrait etre refuse");
        }
    }

    #[test]
    fn un_couple_valide_passe_intact() {
        assert_eq!(
            assainir("KeyA", "KeyD"),
            ("KeyA".to_string(), "KeyD".to_string())
        );
    }

    #[test]
    fn une_touche_invalide_reprend_son_defaut_sans_toucher_a_l_autre() {
        assert_eq!(
            assainir("pas un code", "KeyD"),
            (PRECEDENT_DEFAUT.to_string(), "KeyD".to_string())
        );
        assert_eq!(
            assainir("KeyA", "Tab"),
            ("KeyA".to_string(), SUIVANT_DEFAUT.to_string())
        );
    }

    #[test]
    fn deux_fois_la_meme_touche_rend_la_paire_par_defaut() {
        // Une touche qui irait a la fois a gauche et a droite ne menerait
        // nulle part. On ne devine pas laquelle garder.
        assert_eq!(
            assainir("KeyA", "KeyA"),
            (PRECEDENT_DEFAUT.to_string(), SUIVANT_DEFAUT.to_string())
        );
        // Y compris quand la collision nait de l'assainissement lui meme.
        assert_eq!(
            assainir("KeyQ", "Tab"),
            (PRECEDENT_DEFAUT.to_string(), SUIVANT_DEFAUT.to_string())
        );
    }

    #[test]
    fn l_etiquette_de_repli_reste_lisible() {
        assert_eq!(etiquette_de_repli("KeyQ"), "Q");
        assert_eq!(etiquette_de_repli("Digit4"), "4");
        assert_eq!(etiquette_de_repli("ArrowLeft"), "←");
        assert_eq!(etiquette_de_repli("Comma"), ",");
        // Un code qu'on ne sait pas raccourcir se montre tel quel plutot
        // que de laisser une pastille vide.
        assert_eq!(etiquette_de_repli("F5"), "F5");
    }
}

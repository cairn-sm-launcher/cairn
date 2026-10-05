//! Les codes de mise en forme de Trackmania.
//!
//! Les noms de serveurs, de joueurs et de titres portent des codes introduits
//! par `$`. Affiches tels quels ils donnent `$o$nNac's $i$63EBedroom` au lieu
//! de `Nac's Bedroom`. Ce module les retire.
//!
//! Il sert deux usages. [`nettoyer`] rend le texte nu, qui est ce sur
//! quoi on cherche, on trie et on compare. [`segments`] rend le texte
//! decoupe avec sa mise en forme, qui est ce qu'on affiche depuis le
//! lot 20. Les deux sortent du meme automate, donc ils ne peuvent pas
//! diverger.
//!
//! La lisibilite des couleurs, elle, n'est pas ici : voir
//! [`crate::lisibilite`]. Ce module dit ce que le jeu a ecrit, l'autre
//! dit comment l'afficher sans que ce soit invisible.

/// Vrai pour un caractere de la zone a usage prive d'Unicode.
///
/// **Pourquoi Cairn les jette.** ManiaPlanet embarque une police maison
/// dont les icones, drapeaux, coupes, logos de titres, vivent dans cette
/// zone. Un nom de serveur en contient souvent une ou deux. Dans le jeu
/// elles se dessinent. Partout ailleurs, et donc chez nous, elles
/// tombent sur le carre vide que le navigateur met quand aucune police
/// ne connait le caractere.
///
/// Trois issues possibles, une seule tenable. Embarquer la police de
/// Nadeo n'est pas a nous : Cairn est ouvert, et redistribuer une police
/// proprietaire est exactement le genre de porte qu'on ne laisse pas
/// ouverte. Traduire chaque code en une icone equivalente demanderait
/// une table que personne ne publie, et une traduction fausse serait
/// pire que rien. Reste a les retirer : le nom perd une decoration qu'on
/// ne pouvait de toute facon pas montrer, et il devient lisible.
///
/// Les trois zones sont couvertes, la principale et les deux
/// supplementaires. La frontiere haute compte : `U+F8FF` est le dernier
/// caractere prive, `U+F900` est un ideogramme de compatibilite, un vrai
/// caractere qui doit survivre.
pub fn est_glyphe_prive(c: char) -> bool {
    matches!(c,
        '\u{E000}'..='\u{F8FF}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}')
}

/// Un morceau de texte, avec la couleur en vigueur au moment ou il est ecrit.
///
/// La couleur est celle du dernier code `$RGB` rencontre, `None` si aucun ou
/// si un `$z` l'a remise a zero. Les trois chiffres hexadecimaux du jeu sont
/// etendus a six par duplication de quartet, comme le fait le CSS pour
/// `#abc` : `$63E` donne `#6633EE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub texte: String,
    pub couleur: Option<[u8; 3]>,
    /// `$o`, jusqu'au prochain `$z`.
    pub gras: bool,
    /// `$i`, jusqu'au prochain `$z`.
    pub italique: bool,
}

/// Un segment pret a afficher : la couleur y est deja corrigee et ecrite
/// sous la forme que veut un attribut `style`.
///
/// **Ce que Cairn rend et ce qu'il laisse tomber.** La couleur, le gras
/// et l'italique passent : ils portent l'identite d'un serveur et ils ne
/// changent pas la place que le texte occupe. `$w` et `$n`, large et
/// etroit, changent les metriques et feraient respirer ou deborder une
/// rangee dont toute la valeur est d'avoir la meme grille partout. `$s`,
/// l'ombre, brouille du texte de treize pixels. `$t` force les
/// majuscules, donc il reecrit le nom, et un nom qu'on affiche autrement
/// qu'il n'est ecrit se cherche mal. Les trois sont avales, comme avant.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SegmentAffiche {
    pub texte: String,
    /// `None` quand le segment prend la couleur ordinaire de l'interface.
    pub couleur: Option<String>,
    pub gras: bool,
    pub italique: bool,
}

/// Decoupe pour l'affichage, couleurs deja ramenees a un contraste
/// lisible sur le fond de Cairn.
///
/// Les blancs de bord sautent et les blancs doubles se resserrent,
/// exactement comme dans [`nettoyer`], puisque les deux passent par le
/// meme [`normaliser_espaces`]. Un nom qui commence par un code suivi
/// d'un espace est frequent, et cet espace decalerait la colonne d'une
/// ligne sur l'autre.
pub fn segments_affiches(texte: &str) -> Vec<SegmentAffiche> {
    segments(texte)
        .into_iter()
        .map(|segment| SegmentAffiche {
            texte: segment.texte,
            couleur: segment
                .couleur
                .map(|c| crate::lisibilite::en_hexadecimal(crate::lisibilite::lisible(c))),
            gras: segment.gras,
            italique: segment.italique,
        })
        .collect()
}

/// Retire tous les codes de mise en forme et rend le texte nu.
///
/// Un `$$` est un dollar litteral et survit. Le resultat est debarrasse de ses
/// espaces de bord, parce qu'un nom qui commence par un code suivi d'un espace
/// est frequent et que l'espace n'etait la que pour separer du code.
pub fn nettoyer(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for segment in segments(texte) {
        sortie.push_str(&segment.texte);
    }
    sortie.trim().to_string()
}

/// Decoupe le texte en morceaux, en suivant la couleur courante.
///
/// L'automate est plat : le jeu n'imbrique pas ses codes, un `$RGB` remplace
/// simplement le precedent et `$z` remet tout a zero. Inutile d'empiler.
pub fn segments(texte: &str) -> Vec<Segment> {
    let caracteres: Vec<char> = texte.chars().collect();
    let mut resultat: Vec<Segment> = Vec::new();
    let mut courant = String::new();
    let mut couleur: Option<[u8; 3]> = None;
    let mut gras = false;
    let mut italique = false;
    let mut i = 0usize;

    // Ferme le morceau en cours et l'ajoute, s'il contient quelque chose.
    fn pousser(
        resultat: &mut Vec<Segment>,
        courant: &mut String,
        couleur: Option<[u8; 3]>,
        gras: bool,
        italique: bool,
    ) {
        if !courant.is_empty() {
            resultat.push(Segment {
                texte: std::mem::take(courant),
                couleur,
                gras,
                italique,
            });
        }
    }

    while i < caracteres.len() {
        let c = caracteres[i];
        if c != '$' {
            // Les icones de la police de Nadeo ne se dessinent nulle
            // part ailleurs que dans le jeu, voir `est_glyphe_prive`.
            if !est_glyphe_prive(c) {
                courant.push(c);
            }
            i += 1;
            continue;
        }

        // Un `$` en toute fin de chaine n'introduit rien, c'est un caractere.
        let Some(&suivant) = caracteres.get(i + 1) else {
            courant.push('$');
            i += 1;
            continue;
        };

        // `$$` : dollar litteral.
        if suivant == '$' {
            courant.push('$');
            i += 2;
            continue;
        }

        // `$RGB` : trois chiffres hexadecimaux.
        if let Some(rvb) = lire_couleur(&caracteres, i + 1) {
            pousser(&mut resultat, &mut courant, couleur, gras, italique);
            couleur = Some(rvb);
            i += 4;
            continue;
        }

        let minuscule = suivant.to_ascii_lowercase();

        // `$l[cible]`, `$h[cible]`, `$p[cible]` : un lien avec sa cible entre
        // crochets. La cible n'est pas du texte affichable, elle disparait.
        // Ces trois codes existent aussi sans crochets, auquel cas ils
        // tombent dans le cas general plus bas.
        if matches!(minuscule, 'l' | 'h' | 'p') {
            if let Some(fin) = fin_de_crochets(&caracteres, i + 2) {
                i = fin + 1;
                continue;
            }
        }

        // `$z` remet la mise en forme a zero, couleur comprise.
        if minuscule == 'z' {
            pousser(&mut resultat, &mut courant, couleur, gras, italique);
            couleur = None;
            gras = false;
            italique = false;
            i += 2;
            continue;
        }

        // `$g` rend la couleur ordinaire sans toucher au reste.
        if minuscule == 'g' {
            pousser(&mut resultat, &mut courant, couleur, gras, italique);
            couleur = None;
            i += 2;
            continue;
        }

        // `$o` et `$i` : gras et italique. Ce sont des interrupteurs qui
        // s'allument et que seul `$z` eteint, et pas des bascules : le
        // jeu se comporte ainsi, et un nom qui porte deux fois `$o` ne
        // doit pas redevenir maigre.
        if matches!(minuscule, 'o' | 'i') {
            let deja = if minuscule == 'o' { gras } else { italique };
            if !deja {
                pousser(&mut resultat, &mut courant, couleur, gras, italique);
                if minuscule == 'o' {
                    gras = true;
                } else {
                    italique = true;
                }
            }
            i += 2;
            continue;
        }

        // Tous les autres codes d'une lettre : etroit, large, ombre,
        // majuscules forcees, et les quelques uns qui ne servent qu'au
        // jeu. On les avale sans rien afficher, voir [`SegmentAffiche`].
        if minuscule.is_ascii_alphabetic() || matches!(suivant, '<' | '>' | '[' | ']') {
            i += 2;
            continue;
        }

        // `$` suivi d'autre chose : ce n'etait pas un code.
        courant.push('$');
        i += 1;
    }

    pousser(&mut resultat, &mut courant, couleur, gras, italique);
    normaliser_espaces(&mut resultat);
    resultat
}

/// Resserre les blancs et taille les bords, sur la liste entiere.
///
/// Deux raisons de le faire ici plutot que dans chaque appelant. La
/// premiere est qu'un code ou une icone retiree laisse deux espaces qui
/// se suivent, et que ces deux espaces peuvent se trouver dans deux
/// morceaux differents : il faut voir la suite pour les resserrer. La
/// seconde est que [`nettoyer`] et [`segments_affiches`] doivent rendre
/// exactement le meme texte, ce qu'un test verifie, et le seul moyen sur
/// est qu'ils partagent ce passage.
///
/// **Ce que ca change au passage :** deux espaces voulus dans un nom
/// deviennent un seul. C'est assume. Dans une colonne d'une seule ligne
/// qui se termine en points de suspension, personne ne peut voir la
/// difference, et le cas est bien plus rare que celui qu'on repare.
fn normaliser_espaces(segments: &mut Vec<Segment>) {
    let mut espace_en_attente = false;
    let mut vu_du_texte = false;
    for segment in segments.iter_mut() {
        let mut sortie = String::with_capacity(segment.texte.len());
        for c in segment.texte.chars() {
            if c.is_whitespace() {
                // Un blanc de tete, avant toute lettre, ne sert a rien.
                espace_en_attente = vu_du_texte;
                continue;
            }
            if espace_en_attente {
                sortie.push(' ');
                espace_en_attente = false;
            }
            sortie.push(c);
            vu_du_texte = true;
        }
        segment.texte = sortie;
    }
    // `espace_en_attente` reste vrai s'il y avait un blanc final : on ne
    // le pose jamais, donc la fin est taillee sans autre traitement.
    segments.retain(|s| !s.texte.is_empty());
}

/// Lit trois chiffres hexadecimaux a partir de `depart` et les etend a six.
fn lire_couleur(caracteres: &[char], depart: usize) -> Option<[u8; 3]> {
    let mut composantes = [0u8; 3];
    for (rang, composante) in composantes.iter_mut().enumerate() {
        let quartet = caracteres.get(depart + rang)?.to_digit(16)? as u8;
        // Duplication de quartet : 6 devient 0x66, comme `#abc` en CSS.
        *composante = quartet * 17;
    }
    Some(composantes)
}

/// Position du `]` fermant si `depart` tombe sur un `[`, sinon `None`.
fn fin_de_crochets(caracteres: &[char], depart: usize) -> Option<usize> {
    if caracteres.get(depart) != Some(&'[') {
        return None;
    }
    (depart + 1..caracteres.len()).find(|&j| caracteres[j] == ']')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_sans_code_ne_bouge_pas() {
        assert_eq!(nettoyer("Obstacle Server"), "Obstacle Server");
    }

    #[test]
    fn les_cas_reels_releves_dans_la_liste_des_serveurs() {
        assert_eq!(nettoyer("$o$nNac's $i$63EBedroom"), "Nac's Bedroom");
        assert_eq!(nettoyer("$fffELITE $f00OFFICIAL"), "ELITE OFFICIAL");
        assert_eq!(nettoyer("$s$t$wRoyal"), "Royal");
    }

    #[test]
    fn le_dollar_double_est_un_dollar_litteral() {
        assert_eq!(nettoyer("100$$ Server"), "100$ Server");
        // Et il ne doit surtout pas relancer l'analyse : ici le `o` reste.
        assert_eq!(nettoyer("$$obstacle"), "$obstacle");
    }

    #[test]
    fn les_liens_perdent_leur_cible() {
        assert_eq!(nettoyer("$l[http://ex.fr]Site$l"), "Site");
        assert_eq!(nettoyer("$h[#qjoin=a@b]Rejoindre$h"), "Rejoindre");
        // Sans crochets, le code est un simple interrupteur.
        assert_eq!(nettoyer("$lSite"), "Site");
    }

    #[test]
    fn un_dollar_final_ou_suivi_d_un_chiffre_reste_du_texte() {
        assert_eq!(nettoyer("Prix 5$"), "Prix 5$");
        // `$1` n'est pas une couleur, il manque deux chiffres.
        assert_eq!(nettoyer("a$1b"), "a$1b");
    }

    #[test]
    fn un_nom_entierement_code_devient_vide() {
        assert_eq!(nettoyer("$fff$o$i$n"), "");
    }

    #[test]
    fn la_casse_des_codes_est_sans_importance() {
        assert_eq!(nettoyer("$O$Ngras"), "gras");
        assert_eq!(nettoyer("$FFFblanc"), "blanc");
    }

    #[test]
    fn les_couleurs_sont_etendues_par_duplication_de_quartet() {
        let s = segments("$63EBedroom");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].texte, "Bedroom");
        assert_eq!(s[0].couleur, Some([0x66, 0x33, 0xEE]));
    }

    #[test]
    fn la_remise_a_zero_efface_la_couleur() {
        let s = segments("$f00rouge$znormal");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].couleur, Some([0xFF, 0x00, 0x00]));
        assert_eq!(s[1].couleur, None);
        assert_eq!(s[1].texte, "normal");
    }

    #[test]
    fn les_accents_et_les_emojis_ne_sont_pas_coupes() {
        // Le decoupage se fait sur des caracteres, pas sur des octets : un
        // index sur les octets couperait ces chaines au milieu d'un
        // caractere et ferait paniquer le programme.
        assert_eq!(nettoyer("$o Équipe Française"), "Équipe Française");
        assert_eq!(nettoyer("$f00🎮 Arena"), "🎮 Arena");
    }

    #[test]
    fn le_vide_reste_vide() {
        assert_eq!(nettoyer(""), "");
        assert_eq!(nettoyer("   "), "");
    }

    #[test]
    fn le_gras_et_l_italique_sont_retenus() {
        let s = segments("$onormal$iet penche");
        assert_eq!(s.len(), 2);
        assert!(s[0].gras && !s[0].italique);
        assert!(s[1].gras && s[1].italique);
    }

    #[test]
    fn un_code_de_style_repete_ne_l_eteint_pas() {
        // Le jeu traite `$o` comme un interrupteur qu'on allume, pas
        // comme une bascule. Deux `$o` laissent le texte gras, et ils ne
        // doivent pas non plus couper le segment en deux pour rien.
        let s = segments("$ogras$oencore");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].texte, "grasencore");
        assert!(s[0].gras);
    }

    #[test]
    fn la_remise_a_zero_eteint_aussi_les_styles() {
        let s = segments("$o$igras penche$znu");
        assert_eq!(s.len(), 2);
        assert!(s[0].gras && s[0].italique);
        assert!(!s[1].gras && !s[1].italique);
    }

    #[test]
    fn le_code_g_rend_la_couleur_ordinaire_sans_toucher_au_gras() {
        let s = segments("$o$f00rouge gras$gnoir gras");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].couleur, Some([0xFF, 0, 0]));
        assert_eq!(s[1].couleur, None);
        assert!(s[0].gras && s[1].gras);
    }

    #[test]
    fn les_segments_affiches_portent_une_couleur_lisible() {
        // Un nom en noir sur notre fond sombre : la couleur doit remonter.
        let s = segments_affiches("$000invisible");
        assert_eq!(s.len(), 1);
        let couleur = s[0].couleur.clone().unwrap();
        assert_ne!(couleur, "#000000");
        assert!(couleur.starts_with('#') && couleur.len() == 7);

        // Un nom sans code n'a pas de couleur du tout : il prend celle de
        // l'interface, et c'est ce qu'on veut pour l'immense majorite.
        let s = segments_affiches("Obstacle Server");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].couleur, None);
    }

    #[test]
    fn le_texte_affiche_et_le_texte_nu_disent_la_meme_chose() {
        // Les deux sortent du meme automate, donc ils ne peuvent pas
        // diverger. Ce test est la pour que ca reste vrai.
        for brut in [
            "$o$nNac's $i$63EBedroom",
            "$fffELITE $f00OFFICIAL",
            "$l[http://ex.fr]Site$l",
            "100$$ Server",
            "$f00🎮 Arena",
            "$o  Espaces de bord  ",
            "$fff$o$i$n",
        ] {
            let colle: String = segments_affiches(brut)
                .into_iter()
                .map(|s| s.texte)
                .collect();
            assert_eq!(colle, nettoyer(brut));
        }
    }

    #[test]
    fn les_icones_de_la_police_de_nadeo_disparaissent() {
        // Le cas reel, releve sur les WebServices le 2026-09-25.
        assert_eq!(
            nettoyer("$o$nNac's $i$63EBedroom \u{f186}"),
            "Nac's Bedroom"
        );
        assert_eq!(
            nettoyer("$o$3df \u{f038} $88fMa$9cfx$3df \u{f037} $88fMa$9cfp$3df \u{f036}"),
            "Max Map"
        );
    }

    #[test]
    fn les_trois_zones_privees_sont_couvertes() {
        for prive in [
            '\u{E000}',
            '\u{F8FF}',
            '\u{F0000}',
            '\u{FFFFD}',
            '\u{100000}',
        ] {
            assert!(est_glyphe_prive(prive), "{prive:?} devrait etre prive");
        }
    }

    #[test]
    fn la_frontiere_haute_ne_mange_pas_de_vrais_caracteres() {
        // U+F900 est un ideogramme de compatibilite, juste apres la zone
        // privee. Il doit survivre, comme le lambda grec et la marque
        // d'iteration japonaise qu'on trouve dans de vrais noms.
        for vrai in ['\u{F900}', '\u{39B}', '\u{3005}', 'é', '🎮'] {
            assert!(!est_glyphe_prive(vrai), "{vrai:?} n'est pas prive");
        }
        assert_eq!(
            nettoyer("\u{39B}tría Yox\u{3005}"),
            "\u{39B}tría Yox\u{3005}"
        );
    }

    #[test]
    fn un_nom_fait_de_seules_icones_devient_vide() {
        assert_eq!(nettoyer("\u{f038}\u{f037}\u{f036}"), "");
        // Et le decoupage aussi, ce qui fait retomber le serveur sur son
        // login, voir `serveurs::convertir_un`.
        assert!(segments_affiches("$fff\u{f038} \u{f037}").is_empty());
    }

    #[test]
    fn les_blancs_doubles_se_resserrent() {
        // Ce qu'une icone retiree laisse derriere elle.
        assert_eq!(nettoyer("Max \u{f037} Map"), "Max Map");
        // Y compris quand les deux espaces tombent dans deux morceaux
        // differents, de part et d'autre d'un changement de couleur.
        let s = segments_affiches("$f00Max $0f0\u{f037} $00fMap");
        let colle: String = s.iter().map(|m| m.texte.as_str()).collect();
        assert_eq!(colle, "Max Map");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn le_blanc_de_tete_et_de_queue_saute() {
        assert_eq!(nettoyer("$o   Équipe   "), "Équipe");
        let s = segments_affiches("$o  Équipe Française  ");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].texte, "Équipe Française");
    }
}

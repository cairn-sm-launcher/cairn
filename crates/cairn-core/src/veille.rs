//! La veille : ce que Cairn regarde entre deux gestes de l'utilisateur.
//!
//! Une seule chose bouge sans qu'on y touche : le jeu s'ouvre ou se
//! ferme, et c'est elle qui decide du chemin de lancement. Jusqu'au lot
//! 19d l'interface redemandait pour autant l'etat **complet** toutes les
//! cinq secondes : le registre, la liste des processus, le dossier des
//! profils et deux lignes de journal a chaque fois. Flow l'a dit sans
//! detour, et il a raison : c'est propice aux lags et aux problemes
//! futurs.
//!
//! D'ou ce module. Il tient la difference entre **regarder** et
//! **relever**. Regarder coute une liste de processus et une date de
//! dossier. Relever coute tout le reste, et n'a lieu que quand regarder
//! a vu quelque chose changer.
//!
//! Rien ici ne touche au disque : l'empreinte est fabriquee par
//! `cairn-app`, ce module ne fait que la comparer.

/// Ce qu'un coup d'oeil rapporte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Empreinte {
    /// Le jeu tourne-t-il en ce moment.
    pub jeu_lance: bool,
    /// De quoi reconnaitre le dossier des profils sans le relire : la
    /// date de modification la plus recente qu'on y trouve, et le
    /// nombre de fichiers. `None` quand il n'y a pas de dossier.
    ///
    /// La date seule ne suffirait pas : un profil supprime ne rajeunit
    /// pas le dossier, et son entree resterait dans la liste.
    pub profils: Option<(u64, usize)>,
}

/// Ce qu'il faut faire de ce coup d'oeil.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Veille {
    pub jeu_lance: bool,
    /// Vrai quand le dossier des profils a bouge, donc quand il faut
    /// aller chercher l'etat complet.
    pub profils_a_relire: bool,
    /// Vrai quand le jeu vient de s'ouvrir ou de se fermer. C'est le
    /// seul moment ou cette information merite une ligne de journal :
    /// jusqu'ici elle en ecrivait une toutes les cinq secondes pour
    /// redire la meme chose.
    pub jeu_a_change: bool,
}

/// Compare un coup d'oeil au precedent.
///
/// `avant` est `None` au tout premier, juste apres le releve complet du
/// demarrage : rien n'a alors change, sinon l'interface referait
/// immediatement le travail qu'elle vient de faire.
pub fn comparer(avant: Option<&Empreinte>, maintenant: &Empreinte) -> Veille {
    match avant {
        None => Veille {
            jeu_lance: maintenant.jeu_lance,
            profils_a_relire: false,
            jeu_a_change: false,
        },
        Some(avant) => Veille {
            jeu_lance: maintenant.jeu_lance,
            profils_a_relire: avant.profils != maintenant.profils,
            jeu_a_change: avant.jeu_lance != maintenant.jeu_lance,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empreinte(jeu_lance: bool, profils: Option<(u64, usize)>) -> Empreinte {
        Empreinte { jeu_lance, profils }
    }

    #[test]
    fn le_premier_coup_d_oeil_ne_declenche_rien() {
        let v = comparer(None, &empreinte(true, Some((10, 2))));
        assert!(v.jeu_lance);
        assert!(!v.profils_a_relire);
        assert!(!v.jeu_a_change);
    }

    #[test]
    fn deux_coups_d_oeil_identiques_ne_declenchent_rien() {
        let e = empreinte(false, Some((10, 2)));
        let v = comparer(Some(&e), &e.clone());
        assert!(!v.profils_a_relire);
        assert!(!v.jeu_a_change);
    }

    #[test]
    fn le_jeu_qui_s_ouvre_puis_se_ferme_se_voit_les_deux_fois() {
        let ferme = empreinte(false, Some((10, 2)));
        let ouvert = empreinte(true, Some((10, 2)));
        assert!(comparer(Some(&ferme), &ouvert).jeu_a_change);
        assert!(comparer(Some(&ouvert), &ferme).jeu_a_change);
        // Et il ne fait pas relire les profils au passage.
        assert!(!comparer(Some(&ferme), &ouvert).profils_a_relire);
    }

    #[test]
    fn un_profil_utilise_ajoute_ou_supprime_fait_relire() {
        let depart = empreinte(false, Some((10, 2)));
        // Utilise : la date bouge, le nombre non.
        assert!(comparer(Some(&depart), &empreinte(false, Some((11, 2)))).profils_a_relire);
        // Ajoute.
        assert!(comparer(Some(&depart), &empreinte(false, Some((11, 3)))).profils_a_relire);
        // Supprime : la date ne rajeunit pas, seul le nombre le dit.
        assert!(comparer(Some(&depart), &empreinte(false, Some((10, 1)))).profils_a_relire);
        // Le dossier disparu compte aussi.
        assert!(comparer(Some(&depart), &empreinte(false, None)).profils_a_relire);
    }
}

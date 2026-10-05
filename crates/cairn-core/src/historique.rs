//! L'historique de navigation.
//!
//! **Pourquoi il existe.** Jusqu'ici les boutons lateraux de la souris
//! faisaient exactement ce que font les deux touches de navigation :
//! parcourir les trois onglets. Flow l'a releve, et c'est un doublon
//! sans interet. Precedent et Suivant, sur une souris, veulent dire une
//! seule chose pour tout le monde, et c'est ce que fait un navigateur :
//! revenir la ou on etait.
//!
//! La difference se voit des qu'on sort des trois onglets. Jouer, puis
//! Parcourir les serveurs, puis Reglages : Precedent ramene a Parcourir
//! et non a Jouer. Les deux touches ne savent pas faire ca, elles ne
//! connaissent que la rangee d'onglets.
//!
//! **Ce qui compte ici est ce qui n'entre PAS dans la pile.** Un
//! sous-onglet de Reglages, un groupe qu'on deplie, un filtre qu'on
//! pose : ce sont des commandes dans un ecran, pas des ecrans. Les
//! empiler ferait de Precedent une touche « annuler », et plus personne
//! ne saurait ce qu'elle va faire. La pile ne retient donc que les
//! ecrans, et c'est l'appelant qui decide de ce qui en est un.
//!
//! Rien ici ne touche au disque : l'historique meurt avec la fenetre,
//! comme celui d'un onglet de navigateur qu'on ferme.

/// Le nombre d'entrees gardees.
///
/// Largement au dela de ce qu'un humain remonte a la main, et borne
/// quand meme : une pile sans fin dans une application qui reste ouverte
/// des heures n'a aucune raison d'exister.
pub const PROFONDEUR: usize = 50;

/// Ce que l'interface a besoin de savoir apres chaque mouvement.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EtatNavigation {
    pub ecran: String,
    /// De quoi eteindre la fleche quand il n'y a rien derriere, plutot
    /// que de la faire apparaitre et disparaitre : une commande qui
    /// change de place est plus derangeante qu'une commande eteinte.
    pub peut_reculer: bool,
    pub peut_avancer: bool,
}

#[derive(Debug, Clone)]
pub struct Historique {
    entrees: Vec<String>,
    /// L'index de l'ecran courant dans `entrees`. Toujours valide :
    /// `entrees` n'est jamais vide.
    position: usize,
}

impl Historique {
    /// Un historique qui commence sur un ecran.
    pub fn depuis(ecran: &str) -> Self {
        Self {
            entrees: vec![ecran.to_string()],
            position: 0,
        }
    }

    pub fn courant(&self) -> &str {
        &self.entrees[self.position]
    }

    pub fn peut_reculer(&self) -> bool {
        self.position > 0
    }

    pub fn peut_avancer(&self) -> bool {
        self.position + 1 < self.entrees.len()
    }

    pub fn etat(&self) -> EtatNavigation {
        EtatNavigation {
            ecran: self.courant().to_string(),
            peut_reculer: self.peut_reculer(),
            peut_avancer: self.peut_avancer(),
        }
    }

    /// Va sur un ecran, et empile.
    ///
    /// Deux regles reprises du navigateur. Aller sur l'ecran ou l'on est
    /// deja n'empile rien, sinon cliquer deux fois sur le meme onglet
    /// remplirait la pile d'entrees identiques qu'il faudrait ensuite
    /// remonter une par une. Et naviguer apres etre revenu en arriere
    /// **coupe la branche d'avant** : on ne peut pas etre a la fois sur
    /// le chemin qu'on vient de reprendre et sur celui qu'on a quitte.
    pub fn naviguer(&mut self, ecran: &str) -> EtatNavigation {
        if self.courant() == ecran {
            return self.etat();
        }
        self.entrees.truncate(self.position + 1);
        self.entrees.push(ecran.to_string());
        // La borne se prend par le debut : ce qu'on jette est le plus
        // ancien, donc le moins susceptible d'etre redemande.
        if self.entrees.len() > PROFONDEUR {
            let trop = self.entrees.len() - PROFONDEUR;
            self.entrees.drain(0..trop);
        }
        self.position = self.entrees.len() - 1;
        self.etat()
    }

    /// Revient a l'ecran precedent. Sans effet au bout de la pile.
    pub fn reculer(&mut self) -> EtatNavigation {
        if self.peut_reculer() {
            self.position -= 1;
        }
        self.etat()
    }

    /// Repart vers l'avant, apres un retour. Sans effet s'il n'y a rien.
    pub fn avancer(&mut self) -> EtatNavigation {
        if self.peut_avancer() {
            self.position += 1;
        }
        self.etat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chemin(h: &Historique) -> Vec<&str> {
        h.entrees.iter().map(String::as_str).collect()
    }

    #[test]
    fn un_historique_neuf_ne_mene_nulle_part() {
        let h = Historique::depuis("play");
        assert_eq!(h.courant(), "play");
        assert!(!h.peut_reculer());
        assert!(!h.peut_avancer());
    }

    #[test]
    fn precedent_remonte_le_chemin_reellement_parcouru() {
        let mut h = Historique::depuis("play");
        h.naviguer("servers");
        h.naviguer("appsettings");
        assert_eq!(h.reculer().ecran, "servers");
        assert_eq!(h.reculer().ecran, "play");
        // Et pas au dela.
        assert_eq!(h.reculer().ecran, "play");
    }

    #[test]
    fn suivant_repart_vers_l_avant() {
        let mut h = Historique::depuis("play");
        h.naviguer("edit");
        h.reculer();
        let etat = h.avancer();
        assert_eq!(etat.ecran, "edit");
        assert!(!etat.peut_avancer);
    }

    #[test]
    fn naviguer_apres_un_retour_coupe_la_branche_d_avant() {
        let mut h = Historique::depuis("play");
        h.naviguer("edit");
        h.naviguer("community");
        h.reculer();
        let etat = h.naviguer("appsettings");
        assert_eq!(chemin(&h), vec!["play", "edit", "appsettings"]);
        assert!(etat.peut_reculer);
        assert!(!etat.peut_avancer);
    }

    #[test]
    fn revenir_sur_l_ecran_courant_n_empile_rien() {
        let mut h = Historique::depuis("play");
        h.naviguer("edit");
        h.naviguer("edit");
        h.naviguer("edit");
        assert_eq!(chemin(&h), vec!["play", "edit"]);
        // Un seul retour suffit donc a revenir vraiment en arriere.
        assert_eq!(h.reculer().ecran, "play");
    }

    #[test]
    fn un_aller_retour_repete_ne_gonfle_pas_la_pile() {
        let mut h = Historique::depuis("play");
        for _ in 0..10 {
            h.naviguer("servers");
            h.reculer();
        }
        assert_eq!(chemin(&h), vec!["play", "servers"]);
        assert_eq!(h.courant(), "play");
    }

    #[test]
    fn la_pile_est_bornee_et_jette_le_plus_ancien() {
        let mut h = Historique::depuis("ecran-0");
        for i in 1..PROFONDEUR + 20 {
            h.naviguer(&format!("ecran-{i}"));
        }
        assert_eq!(chemin(&h).len(), PROFONDEUR);
        assert_eq!(h.courant(), &format!("ecran-{}", PROFONDEUR + 19));
        // Le plus ancien est parti, le plus recent est la.
        assert!(!chemin(&h).contains(&"ecran-0"));
        assert!(h.peut_reculer());
    }

    #[test]
    fn l_etat_dit_ce_que_les_fleches_doivent_montrer() {
        let mut h = Historique::depuis("play");
        assert_eq!(
            h.etat(),
            EtatNavigation {
                ecran: "play".to_string(),
                peut_reculer: false,
                peut_avancer: false,
            }
        );
        h.naviguer("servers");
        let etat = h.etat();
        assert!(etat.peut_reculer);
        assert!(!etat.peut_avancer);
        let etat = h.reculer();
        assert!(!etat.peut_reculer);
        assert!(etat.peut_avancer);
    }
}

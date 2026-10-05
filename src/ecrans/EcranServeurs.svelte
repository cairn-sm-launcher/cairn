<!--
  Parcourir les serveurs.

  Tout le filtrage, le tri, le groupement et le tirage au sort vivent
  dans `cairn-core`, ou ils sont testes. Cet ecran ne fait que demander
  une vue et la dessiner. C'est pour ca qu'il appelle Rust a chaque
  frappe : la liste est gardee en memoire cote Rust, donc il n'y a ni
  reseau ni duplication de logique, et une regle de tri ne peut pas
  diverger entre deux langages.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import BoutonIcone from '../composants/BoutonIcone.svelte';
  import LigneServeur from '../composants/LigneServeur.svelte';
  import { t } from '../lib/traduction.svelte';
  import {
    reglages,
    systeme,
    message,
    enregistrer,
    basculerFavori,
  } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';

  const accent = $derived(reglages.accent);
  /** L'ambre des favoris, la meme que celle de la tuile Favorites. */
  const OR = '#FFC857';

  let recherche = $state('');
  let vue = $state<pont.Vue | null>(null);
  let chargement = $state(true);
  let erreur = $state<string | null>(null);
  let ouverte = $state<string | null>(null);
  let tiree = $state<string | null>(null);
  /** Les derniers tirages, pour ne pas reproposer les memes. */
  let recents = $state<string[]>([]);
  /**
   * La liste complete des modes est-elle depliee.
   *
   * Volontairement non retenue d'une session a l'autre : ce qui est
   * derriere « +N autres » depend de ce qui est en ligne ce soir la, et
   * rouvrir Cairn sur vingt puces serait une regression par rapport au
   * probleme qu'on resout ici.
   */
  let tousLesModes = $state(false);

  /**
   * Combien de puces de mode avant le repli.
   *
   * La communaute cree ses propres modes, et Flow a compte : certains
   * soirs il y en a une quinzaine. Une barre de filtres qui prend trois
   * rangees ne filtre plus rien, elle encombre. Six, parce que les
   * modes arrivent tries du plus joue au moins joue et que les six
   * premiers couvrent la quasi totalite des joueurs en ligne ; le reste
   * est a un clic, et la recherche par mode le trouve aussi.
   */
  const MODES_VISIBLES = 6;

  const comptes = $derived(vue?.comptes ?? []);
  const modesMontres = $derived.by(() => {
    if (tousLesModes) return comptes;
    const tete = comptes.slice(0, MODES_VISIBLES);
    // Le mode retenu est toujours montre, meme s'il est loin dans la
    // liste : un filtre actif qu'on ne voit pas est un piege.
    const retenu = reglages.serveurs_mode;
    if (retenu && !tete.some((c) => c.mode === retenu)) {
      const trouve = comptes.find((c) => c.mode === retenu);
      if (trouve) return [...tete, trouve];
    }
    return tete;
  });
  const modesCaches = $derived(Math.max(0, comptes.length - modesMontres.length));

  /**
   * Le filtre mot de passe.
   *
   * Il manquait, et c'est le quatrieme point du retour de Flow : la
   * liste melangeait les serveurs proteges aux autres, et on ne s'en
   * apercevait qu'au moment ou le jeu reclamait un mot de passe qu'on
   * n'avait pas.
   */
  const MOTS_DE_PASSE: Array<{ id: pont.MotDePasse; libelle: string }> = [
    { id: 'tous', libelle: 'All' },
    { id: 'libres', libelle: 'No password' },
    { id: 'prives', libelle: 'With password' },
  ];

  const TRIS: Array<{ id: pont.Tri; libelle: string }> = [
    { id: 'joueurs', libelle: 'Players' },
    { id: 'nom', libelle: 'Name' },
    { id: 'mode', libelle: 'Mode' },
  ];

  /** Tous les serveurs retenus, les deux groupes a la suite. */
  const retenus = $derived(vue ? [...vue.peuples, ...vue.vides] : []);

  async function rafraichirVue() {
    if (!pont.pontPresent()) {
      chargement = false;
      return;
    }
    try {
      vue = await pont.vueServeurs(
        reglages.serveurs_mode,
        recherche,
        reglages.serveurs_tri,
        reglages.serveurs_tri_croissant,
        reglages.serveurs_mot_de_passe,
        reglages.serveurs_favoris,
      );
      erreur = null;
    } catch (e) {
      erreur = String(e);
    }
  }

  /** Redemande la liste au reseau, puis la vue. */
  async function recharger() {
    chargement = true;
    erreur = null;
    try {
      systeme.serveurs = await pont.serveurs();
      systeme.serveursCharges = true;
      await rafraichirVue();
    } catch (e) {
      erreur = String(e);
    } finally {
      chargement = false;
    }
  }

  // Au premier affichage on se contente de la liste deja rapportee par
  // l'ecran Jouer, s'il y en a une : rouvrir cet ecran ne doit pas
  // refaire un appel reseau a chaque fois.
  $effect(() => {
    if (systeme.serveursCharges) {
      chargement = false;
      rafraichirVue();
    } else {
      recharger();
    }
  });

  // La recherche est temporisee : pas pour la charge, un filtrage sur
  // cinquante lignes est instantane, mais pour que la liste ne saute pas
  // a chaque lettre pendant qu'on tape un mot.
  let attente: ReturnType<typeof setTimeout> | undefined;
  function surRecherche(valeur: string) {
    recherche = valeur;
    clearTimeout(attente);
    attente = setTimeout(rafraichirVue, 140);
  }

  function choisirMode(mode: string) {
    reglages.serveurs_mode = reglages.serveurs_mode === mode ? '' : mode;
    enregistrer();
    rafraichirVue();
  }

  /**
   * Le tri, et son sens.
   *
   * Trois etats par critere : le sens naturel, le sens inverse, puis le
   * retour au tri par defaut. C'est le retour de Flow au lot 19d bis :
   * deux etats laissaient la liste triee pour toujours, sans moyen de
   * revenir a l'ordre d'origine.
   *
   * Le cycle entier vient de Rust, ou il est teste. Il n'y a pas de
   * « liste non triee » a proposer : une liste arrive toujours dans un
   * ordre, et l'etat neutre est celui la, la frequentation decroissante,
   * qui est aussi l'ordre dans lequel l'API rend la liste.
   */
  async function choisirTri(tri: pont.Tri) {
    if (!pont.pontPresent()) return;
    const [suivant, croissant] = await pont.triSuivant(
      reglages.serveurs_tri,
      reglages.serveurs_tri_croissant,
      tri,
    );
    reglages.serveurs_tri = suivant;
    reglages.serveurs_tri_croissant = croissant;
    enregistrer();
    rafraichirVue();
  }

  /** Le filtre mot de passe : tous, ceux qui n'en demandent pas, les autres. */
  function choisirMotDePasse(valeur: pont.MotDePasse) {
    reglages.serveurs_mot_de_passe = valeur;
    enregistrer();
    rafraichirVue();
  }

  function basculerFiltreFavoris() {
    reglages.serveurs_favoris = !reglages.serveurs_favoris;
    enregistrer();
    rafraichirVue();
  }

  async function etoiler(login: string) {
    await basculerFavori(login);
    // La vue depend des favoris des que leur filtre est actif, et le
    // compte de la puce en depend toujours.
    await rafraichirVue();
  }

  function toutEffacer() {
    surRecherche('');
    reglages.serveurs_mot_de_passe = 'tous';
    reglages.serveurs_favoris = false;
    choisirMode('');
  }

  function basculerGroupe(lequel: 'peuples' | 'vides') {
    if (lequel === 'peuples') {
      reglages.serveurs_peuples_ouverts = !reglages.serveurs_peuples_ouverts;
    } else {
      reglages.serveurs_vides_ouverts = !reglages.serveurs_vides_ouverts;
    }
    enregistrer();
  }

  /**
   * Le tirage au sort.
   *
   * Il n'envoie nulle part : il ouvre la ligne tiree, la marque et y fait
   * defiler. Etre teleporte sans savoir ou est desorientant, et si le
   * tirage tombe mal il faut pouvoir voir ce qu'on refuse avant de
   * refuser. Le bouton porte aussi le nombre de candidats, ce qui rend
   * les filtres visibles au moment ou ils comptent.
   */
  function tirerAuSort() {
    if (retenus.length === 0 || !vue) return;
    const frais = retenus.filter((s) => !recents.includes(s.login));
    const pioche = frais.length > 0 ? frais : retenus;
    const choisi = pioche[Math.floor(Math.random() * pioche.length)];

    // Trois de memoire : assez pour ne pas retomber sur le meme deux fois
    // de suite, assez peu pour ne pas epuiser une liste de cinq serveurs.
    recents = [choisi.login, ...recents].slice(0, 3);

    // Le groupe qui porte le tire doit etre deplie, sinon la ligne
    // n'existe pas a l'ecran. Le bouton annonce un nombre de candidats,
    // il ne peut pas en tirer un qu'il serait incapable de montrer.
    const dansLesVides = vue.vides.some((s) => s.login === choisi.login);
    if (dansLesVides && !reglages.serveurs_vides_ouverts) {
      reglages.serveurs_vides_ouverts = true;
      enregistrer();
    }
    if (!dansLesVides && !reglages.serveurs_peuples_ouverts) {
      reglages.serveurs_peuples_ouverts = true;
      enregistrer();
    }

    ouverte = choisi.login;
    tiree = choisi.login;
    // Deux images : la premiere deplie le groupe, la seconde pose la
    // ligne ouverte. Defiler avant, c'est defiler vers un vide.
    requestAnimationFrame(() =>
      requestAnimationFrame(() =>
        document
          .querySelector(`[data-serveur="${CSS.escape(choisi.login)}"]`)
          ?.scrollIntoView({ block: 'center', behavior: 'smooth' }),
      ),
    );
  }

  async function rejoindre(serveur: pont.Serveur, spectate: boolean) {
    if (!reglages.profil) {
      message(t('Pick a profile first'));
      return;
    }
    message(
      spectate ? t('Spectating {0}...', serveur.nom) : t('Joining {0}...', serveur.nom),
    );
    try {
      await pont.rejoindre(serveur.login, serveur.titre, reglages.profil, spectate);
    } catch (e) {
      message(String(e));
    }
  }
</script>

<div class="ecran">
  <!-- Plus de fleche de retour propre a cet ecran depuis l'etape 30 :
       il est devenu un onglet, donc le bandeau est au dessus avec sa
       fleche et son bouton Accueil. En garder une seconde aurait donne
       deux retours a l'ecran, a deux endroits, pour deux comportements
       qui n'auraient pas toujours ete les memes. -->
  <header class="pad entete">
    <div class="gauche">
      <div>
        <h1 class="titre display">{t('ShootMania servers')}</h1>
        <p class="compte">
          {#if chargement}
            {t('Loading...')}
          {:else if vue}
            {t('{0} servers, sorted by players', vue.total)}
            {#if vue.retenus !== vue.total}
              <span class="ecart">· {t('{0} shown', vue.retenus)}</span>
            {/if}
          {/if}
        </p>
      </div>
    </div>
    <BoutonIcone nom="refresh" titre={t('Refresh')} {accent} onclick={recharger} />
  </header>

  <div class="pad barre">
    <div class="commandes">
    <label class="champ">
      <span class="loupe"><Icone nom="search" taille={15} /></span>
      <input
        class="txt"
        type="search"
        value={recherche}
        oninput={(e) => surRecherche(e.currentTarget.value)}
        placeholder={t('Search a server')}
        aria-label={t('Search a server')}
      />
    </label>

    <!-- Le filtre mot de passe. Il etait absent jusqu'ici : la liste
         montrait les serveurs proteges au milieu des autres, et on ne
         s'en apercevait qu'au moment ou le jeu reclamait un mot de
         passe qu'on n'avait pas. -->
    <div class="mdp" role="group" aria-label={t('Password filter')}>
      <span class="mdp-icone"><Icone nom="lock" taille={13} epaisseur={2} /></span>
      {#each MOTS_DE_PASSE as option (option.id)}
        {@const actif = reglages.serveurs_mot_de_passe === option.id}
        <button
          class="seg"
          type="button"
          class:actif
          aria-pressed={actif}
          onclick={() => choisirMotDePasse(option.id)}
          style={actif ? `color:${accent};` : ''}
        >
          {t(option.libelle)}
        </button>
      {/each}
    </div>

    <div class="droite">
      <div class="tri" role="group" aria-label={t('Sort by')}>
        <span class="tri-icone"><Icone nom="sort" taille={14} /></span>
        <!-- Une seule phrase d'aide pour le critere actif, et elle dit
             le cycle entier. Choisir la phrase selon l'etat obligerait a
             savoir ici quel est le sens naturel du critere, donc a
             recopier en TypeScript une regle qui vit en Rust. -->
        {#each TRIS as option (option.id)}
          {@const actif = reglages.serveurs_tri === option.id}
          <button
            class="seg"
            type="button"
            class:actif
            aria-pressed={actif}
            title={actif
              ? t('Click again to reverse, then to go back to the default order')
              : t('Sort by {0}', t(option.libelle).toLowerCase())}
            onclick={() => choisirTri(option.id)}
            style={actif ? `color:${accent};` : ''}
          >
            {t(option.libelle)}
            <!-- La fleche ne s'affiche que sur le critere actif : les
                 deux autres n'ont pas de sens tant qu'on ne les a pas
                 choisis, et trois fleches donneraient a lire trois fois
                 la meme information. -->
            {#if actif}
              <span class="sens">
                <Icone
                  nom={reglages.serveurs_tri_croissant ? 'up' : 'down'}
                  taille={12}
                  epaisseur={2.2}
                />
              </span>
            {/if}
          </button>
        {/each}
      </div>

      <button
        class="hasard"
        type="button"
        onclick={tirerAuSort}
        disabled={retenus.length === 0}
        style="border-color:{accent};color:{accent};"
      >
        <Icone nom="shuffle" taille={15} epaisseur={1.8} />
        {t('At random among {0}', retenus.length)}
      </button>
    </div>
    </div>

    <div class="puces">
      <!-- Les favoris ouvrent la rangee : c'est le filtre qu'on utilise
           le plus souvent et celui qui doit se voir le plus. -->
      <button
        class="chip favoris"
        type="button"
        aria-pressed={reglages.serveurs_favoris}
        onclick={basculerFiltreFavoris}
        style={reglages.serveurs_favoris
          ? `border-color:${OR};background:${OR};color:#0A0C0F;`
          : `color:${OR};border-color:rgba(255,200,87,0.35);`}
      >
        <Icone nom="star" taille={13} epaisseur={1.8} plein={reglages.serveurs_favoris} />
        {t('Favorites')}
        <span class="compte-puce">{reglages.favoris.length}</span>
      </button>

      <button
        class="chip"
        type="button"
        onclick={() => choisirMode('')}
        style={reglages.serveurs_mode === ''
          ? `border-color:${accent};background:${accent};color:#0A0C0F;`
          : ''}
      >
        {t('All')}
        <span class="compte-puce">{vue?.retenus ?? 0}</span>
      </button>
      <!-- Le compte sur chaque puce dit ce qu'elle rapporterait, pas ce
           qu'on a deja : sans lui, on tombe sur une liste vide sans
           savoir lequel des deux filtres l'a videe. -->
      {#each modesMontres as c (c.mode)}
        <button
          class="chip"
          type="button"
          onclick={() => choisirMode(c.mode)}
          style={reglages.serveurs_mode === c.mode
            ? `border-color:${accent};background:${accent};color:#0A0C0F;`
            : ''}
        >
          {c.mode}
          <span class="compte-puce">{c.compte}</span>
        </button>
      {/each}
      <!-- La communaute cree ses propres modes : certains soirs il y en
           a quinze, et la barre prenait trois rangees. Les six plus
           joues restent, le reste est a un clic. Le mode retenu est
           toujours montre, meme s'il est loin dans la liste : un filtre
           actif qu'on ne voit pas est un piege. -->
      {#if modesCaches > 0}
        <button class="chip deplier" type="button" onclick={() => (tousLesModes = true)}>
          {t('+{0} more', modesCaches)}
        </button>
      {:else if tousLesModes && comptes.length > MODES_VISIBLES}
        <button class="chip deplier" type="button" onclick={() => (tousLesModes = false)}>
          {t('Show less')}
        </button>
      {/if}
    </div>

  </div>

  <div class="pad liste">
    {#if chargement}
      <p class="etat-liste">{t('Loading the servers...')}</p>
    {:else if erreur}
      <div class="etat-liste erreur">
        <p>{erreur}</p>
        <button class="btn-ghost relancer" type="button" onclick={recharger}>
          {t('Try again')}
        </button>
      </div>
    {:else if vue && vue.total === 0}
      <p class="etat-liste">{t('No ShootMania server is online right now.')}</p>
    {:else if vue && vue.retenus === 0}
      <div class="etat-liste">
        <p>{t('No server matches these filters.')}</p>
        <button
          class="btn-ghost relancer"
          type="button"
          onclick={toutEffacer}
        >
          {t('Clear the filters')}
        </button>
      </div>
    {:else if vue}
      {#snippet groupe(
        titre: string,
        serveurs: pont.Serveur[],
        ouvert: boolean,
        lequel: 'peuples' | 'vides',
        vide: boolean,
      )}
        {#if serveurs.length > 0}
          <button class="groupe" type="button" onclick={() => basculerGroupe(lequel)}>
            <span class="fleche" class:tourne={ouvert}>
              <Icone nom="chev" taille={13} epaisseur={2.2} />
            </span>
            {titre}
          </button>
          {#if ouvert}
            <div class="rangees">
              {#each serveurs as serveur (serveur.login)}
                <div data-serveur={serveur.login}>
                  <LigneServeur
                    {serveur}
                    {accent}
                    {vide}
                    ouverte={ouverte === serveur.login}
                    marquee={tiree === serveur.login}
                    favori={reglages.favoris.includes(serveur.login)}
                    onfavori={() => etoiler(serveur.login)}
                    onbasculer={() => {
                      ouverte = ouverte === serveur.login ? null : serveur.login;
                      tiree = null;
                    }}
                    onrejoindre={(spectate) => rejoindre(serveur, spectate)}
                  />
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      {/snippet}

      {@render groupe(
        t('{0} with players', vue.peuples.length),
        vue.peuples,
        reglages.serveurs_peuples_ouverts,
        'peuples',
        false,
      )}
      {@render groupe(
        t('{0} empty', vue.vides.length),
        vue.vides,
        reglages.serveurs_vides_ouverts,
        'vides',
        true,
      )}
    {/if}
  </div>
</div>

<style>
  .ecran {
    display: flex;
    flex-direction: column;
    height: 100%;
    position: relative;
    z-index: 1;
  }
  .entete {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 22px;
    padding-bottom: 22px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }
  .gauche {
    display: flex;
    align-items: center;
    gap: 15px;
  }
  .titre {
    font-weight: 700;
    font-size: 21px;
    letter-spacing: -0.02em;
    margin: 0;
  }
  .compte {
    font-size: 12px;
    color: #838c96;
    margin: 2px 0 0;
  }
  .ecart {
    color: #6e7681;
  }

  /* Deux rangees plutot qu'une. Les puces de mode suivent ce qui est en
     ligne ce soir la : il y en a parfois deux, parfois dix. Sur une
     seule rangee, elles poussaient le tri et le tirage au sort a la
     ligne des qu'elles etaient nombreuses. */
  .barre {
    padding-top: 16px;
    padding-bottom: 14px;
    flex-shrink: 0;
  }
  .commandes {
    display: flex;
    align-items: center;
    gap: 14px;
    /* Trois groupes de commandes sur une rangee : en fenetre etroite ils
       passent a la ligne plutot que de se comprimer. */
    flex-wrap: wrap;
  }
  .champ {
    position: relative;
    display: flex;
    align-items: center;
    flex: 0 1 260px;
  }
  .loupe {
    position: absolute;
    left: 11px;
    display: flex;
    color: #6e7681;
    pointer-events: none;
  }
  .champ input {
    width: 100%;
    padding: 9px 12px 9px 33px;
    font-size: 12.5px;
  }
  .puces {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 12px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 14px;
    border-radius: 999px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: transparent;
    color: #9aa3ad;
  }
  .compte-puce {
    font-size: 11px;
    font-weight: 600;
    opacity: 0.7;
  }
  /* La puce des favoris porte sa couleur meme eteinte : c'est ce qui la
     distingue des puces de mode au premier coup d'oeil. */
  .favoris {
    gap: 6px;
  }
  .deplier {
    color: #6e7681;
    border-style: dashed;
  }
  .deplier:hover {
    color: #f4f6f8;
  }

  .droite {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
  }
  .tri {
    display: flex;
    align-items: center;
    gap: 2px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9px;
    padding: 3px 8px 3px 9px;
  }
  .tri-icone,
  .mdp-icone {
    display: flex;
    color: #6e7681;
    margin-right: 5px;
  }
  /* Le filtre mot de passe emprunte la forme du groupe de tri : meme
     cadre, meme icone en tete, memes segments. Deux commandes qui font
     la meme chose, choisir parmi trois, n'ont aucune raison de se
     dessiner differemment. */
  .mdp {
    display: flex;
    align-items: center;
    gap: 2px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9px;
    padding: 3px 8px 3px 9px;
  }
  .sens {
    display: inline-flex;
    margin-left: 4px;
  }
  .seg {
    display: inline-flex;
    align-items: center;
    background: transparent;
    border: 0;
    color: #9aa3ad;
    font-size: 12px;
    font-weight: 600;
    padding: 5px 7px;
    border-radius: 6px;
    cursor: pointer;
  }
  .seg:hover {
    color: #f4f6f8;
  }
  .actif {
    background: rgba(255, 255, 255, 0.06);
  }
  .hasard {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-radius: 9px;
    border: 1px solid;
    background: transparent;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .hasard:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .liste {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 36px;
  }
  .groupe {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    background: transparent;
    border: 0;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    color: #838c96;
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 14px 6px 10px;
    cursor: pointer;
    text-align: left;
  }
  .groupe:hover {
    color: #f4f6f8;
  }
  .fleche {
    display: flex;
    transition: transform 0.15s ease;
  }
  .tourne {
    transform: rotate(90deg);
  }
  .rangees {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .etat-liste {
    font-size: 13px;
    color: #9aa3ad;
    padding: 40px 6px;
    text-align: center;
    line-height: 1.6;
  }
  .erreur {
    color: #ffd8cd;
  }
  .relancer {
    margin-top: 14px;
    background: transparent;
    color: #f4f6f8;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 10px;
    padding: 9px 18px;
    font-weight: 600;
    font-size: 12.5px;
    cursor: pointer;
  }
</style>

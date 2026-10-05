<!--
  Le bandeau fusionne du lot 17, reorganise a l'etape 30 : navigation a
  gauche, cinq onglets au centre, et a droite tout ce qui regarde le
  joueur plutot que l'application. 78 px de haut.

  **Pourquoi le profil est passe a droite.** Il etait a gauche, avant la
  rangee d'onglets. Or la gauche d'un bandeau est la ou l'on revient en
  arriere, et la droite est la ou l'on trouve ce qui est a soi : son
  compte, ses notifications, ses messages, ses reglages. Le profil
  appartient au second groupe, pas au premier. Avec Notifications, Chat
  et Reglages a cote de lui, les quatre commandes personnelles sont
  desormais au meme endroit au lieu d'etre coupees en deux par cinq
  onglets.

  Le centrage est le point delicat, et il n'a pas change en passant a
  Svelte. Une rangee centrale en flex:1 entre deux groupes de largeurs
  differentes se centre dans la place qui reste, pas dans la fenetre :
  mesure au lot 17, les onglets tombaient 53 px a gauche du milieu sur
  une fenetre de 1440. Les deux groupes lateraux prennent donc la meme
  part, flex:1 1 0, et la rangee du milieu garde sa taille propre.
-->
<script lang="ts">
  import BoutonIcone from './BoutonIcone.svelte';
  import BoutonProfil from './BoutonProfil.svelte';
  import { t } from '../lib/traduction.svelte';
  import { reglages, ui, systeme, message, naviguer, reculer } from '../lib/etat.svelte';
  import { ONGLETS, ORDRE_ONGLETS, ECRAN_ACCUEIL } from '../lib/constantes';
  import { etiquette } from '../lib/clavier.svelte';
  import { untrack } from 'svelte';

  const accent = $derived(reglages.accent);
  const etatJeu = $derived(
    systeme.jeuLance ? t('ManiaPlanet running') : t('ManiaPlanet closed'),
  );

  // Les pastilles s'eteignent quand elles ne menent nulle part. Sur
  // Jouer, la pastille A est grisee parce qu'on est deja au bord gauche.
  // Aucun element nouveau a ajouter, l'information est deja au bon
  // endroit, et elle previent le rebond au lieu de le subir.
  const rang = $derived(ORDRE_ONGLETS.indexOf(ui.ecran));

  // La pastille montre l'etiquette reelle de la position, demandee au
  // systeme : A sur un clavier francais, Q sur un anglais, et elle suit
  // toute seule si le joueur change de clavier.
  const raccourcis = $derived({
    precedent: etiquette(reglages.touche_precedent),
    suivant: etiquette(reglages.touche_suivant),
  });

  // Le rebond est joue par l'API d'animations plutot que par une classe
  // CSS. Une classe demande d'etre retiree puis reposee pour rejouer,
  // ce qui suppose d'attendre une image, et deux appuis rapproches sur
  // une touche qui ne mene nulle part ne rebondiraient alors qu'une
  // fois. animate() repart de zero a chaque appel, sans condition.
  let rangee: HTMLElement | undefined = $state();

  // Un aller franc, un retour qui depasse un peu, puis le repos. Les
  // accelerations sont portees par chaque segment et non par l'animation
  // entiere : une courbe posee sur l'ensemble ecrase tout le mouvement
  // dans les premieres dizaines de millisecondes, ce qui se lit comme un
  // clignotement et pas comme un rebond. Mesure : la pointe tombait a
  // 10 % de la duree au lieu du tiers.
  const IMAGES: Array<{ dx: number; offset: number; easing?: string }> = [
    { dx: 0, offset: 0, easing: 'cubic-bezier(.32,0,.28,1)' },
    { dx: -8, offset: 0.34, easing: 'cubic-bezier(.4,0,.35,1)' },
    { dx: 3, offset: 0.68, easing: 'cubic-bezier(.4,0,.4,1)' },
    { dx: 0, offset: 1 },
  ];
  const DUREE = 180;

  /**
   * Le cote ou l'on vient de buter, le temps de le montrer.
   *
   * **Le rebond ne peut pas etre le seul accuse de reception.** Il obeit
   * a l'interrupteur des animations et au reglage systeme de reduction de
   * mouvement, donc il se tait chez une partie des utilisateurs. Et quand
   * il se tait, la touche ne faisait plus rien du tout, ce qui est
   * exactement la panne apparente que le rebond existait pour eviter.
   *
   * La pastille du bout s'allume donc aussi. C'est le bon element : elle
   * est deja eteinte pour annoncer que ce cote ne mene nulle part, donc
   * la voir s'allumer au contact parle le meme langage. Et un changement
   * de couleur n'est pas un mouvement, il reste acceptable pour qui a
   * demande moins d'animation.
   */
  let butee = $state<'gauche' | 'droite' | null>(null);
  let minuterieButee: ReturnType<typeof setTimeout> | undefined;
  const DUREE_BUTEE = 420;

  $effect(() => {
    const coup = ui.rebond;
    if (coup === 0 || !rangee) return;
    untrack(() => {
      butee = ui.rebondCote;
      clearTimeout(minuterieButee);
      minuterieButee = setTimeout(() => (butee = null), DUREE_BUTEE);

      if (!reglages.animations) return;
      if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
      const sens = ui.rebondCote === 'gauche' ? 1 : -1;
      rangee!.animate(
        IMAGES.map((image) => ({
          transform: `translateX(${image.dx * sens}px)`,
          offset: image.offset,
          ...(image.easing ? { easing: image.easing } : {}),
        })),
        { duration: DUREE, easing: 'linear' },
      );
    });
  });
  function touchePorte(sens: string): boolean {
    if (rang === -1) return true;
    if (sens === 'precedent') return rang > 0;
    if (sens === 'suivant') return rang < ORDRE_ONGLETS.length - 1;
    return true;
  }

  /** Vrai quand c'est cette pastille la qu'on vient de heurter. */
  function heurtee(sens: string): boolean {
    return (
      (butee === 'gauche' && sens === 'precedent') ||
      (butee === 'droite' && sens === 'suivant')
    );
  }
</script>

<div class="bandeau">
  <div class="pad rangee">
    <div class="head3-left">
      <!-- La fleche est toujours la, eteinte quand il n'y a rien
           derriere. Elle pourrait n'apparaitre qu'au besoin, mais une
           commande qui apparait et disparait deplace ses voisines et se
           fait oublier : personne ne saurait que Cairn sait revenir en
           arriere. Eteinte, elle l'annonce en permanence sans peser.
           Pas de fleche Suivant : elle ne sert que dans les secondes
           qui suivent un retour, et le bouton de souris comme Alt et
           fleche droite sont alors sous la main. -->
      <BoutonIcone
        nom="chevL"
        titre={t('Back')}
        desactive={!ui.peutReculer}
        {accent}
        onclick={reculer}
      />
      <!-- Accueil, a cote de Precedent. Avec cinq onglets, revenir a
           Jouer pouvait demander quatre pas de clavier ou de viser un
           onglet precis. Une commande qui ramene au point de depart en
           un geste vaut mieux, et c'est le couple qu'on trouve dans
           n'importe quel navigateur. -->
      <BoutonIcone
        nom="home"
        titre={t('Home')}
        desactive={ui.ecran === ECRAN_ACCUEIL}
        {accent}
        onclick={() => naviguer(ECRAN_ACCUEIL)}
      />
    </div>

    <nav class="tab3-row" bind:this={rangee}>
      {#each ONGLETS as onglet (onglet.id)}
        <button
          class="tab3 display"
          class:on={ui.ecran === onglet.id}
          type="button"
          onclick={() => naviguer(onglet.id)}
        >
          <span>{t(onglet.libelle)}</span>
          {#if onglet.touche}
            <span
              class="keyhint mono"
              class:eteinte={!touchePorte(onglet.touche)}
              class:heurtee={heurtee(onglet.touche)}
              style={heurtee(onglet.touche)
                ? `opacity:1; color:#F4F6F8; border-color:${accent};`
                : ''}
            >
              {onglet.touche === 'precedent' ? raccourcis.precedent : raccourcis.suivant}
            </span>
          {/if}
          <i style="background:{accent};"></i>
        </button>
      {/each}
    </nav>

    <div class="head3-right">
      <!-- Sur une fenetre etroite il ne reste que la pastille, le titre
           garde donc le libelle a portee de curseur. La pastille dit si
           le jeu tourne vraiment, et c'est cet etat qui decide du chemin
           de lancement. -->
      <div class="gamestate" title={etatJeu}>
        <span
          class="pastille"
          style="background:{systeme.jeuLance ? accent : '#5A6470'};"
        ></span>
        <span class="lbl">{etatJeu}</span>
      </div>

      <BoutonProfil compact />

      <BoutonIcone
        nom="bell"
        titre={t('Notifications')}
        {accent}
        onclick={() => message(t('No new notifications'))}
      />

      <BoutonIcone
        nom="chat"
        titre={t('Chat')}
        actif={ui.ecran === 'chat'}
        {accent}
        onclick={() => naviguer('chat')}
      >
        <span class="puce" style="background:{accent};"></span>
      </BoutonIcone>

      <BoutonIcone
        nom="gear"
        titre={t('Launcher settings')}
        actif={ui.ecran === 'appsettings'}
        {accent}
        onclick={() => naviguer('appsettings')}
      />
    </div>
  </div>
</div>

<style>
  .bandeau {
    flex-shrink: 0;
    position: relative;
    z-index: 2;
  }
  /* **La hauteur est posee, pas subie.** Elle valait 78 px parce que le
     profil, avec sa pastille de 40 px, etait l'element le plus haut de la
     rangee. Le profil est passe a droite en version compacte a l'etape
     30, et le bandeau a perdu 14 px d'un coup, entrainant toutes les
     tuiles avec lui.

     Or 78 px n'est pas un resultat, c'est l'etalon : tout l'espacement de
     l'application est calcule en fractions de `--bandeau`, voir la
     section des densites dans app.css. Le bandeau reclame donc sa hauteur
     au lieu de la recevoir du plus grand de ses enfants, et un futur
     changement de contenu ne pourra plus la faire deriver en silence. */
  .rangee {
    display: flex;
    align-items: center;
    gap: 18px;
    padding-top: 18px;
    padding-bottom: 6px;
    min-height: var(--bandeau, 78px);
  }
  /* Une pastille qui ne mene nulle part s'efface, elle ne disparait pas :
     la place reste prise, donc les onglets ne se deplacent pas quand on
     change d'ecran. */
  .eteinte {
    opacity: 0.32;
    border-color: rgba(255, 255, 255, 0.06);
    transition: opacity 0.15s ease;
  }

  /* L'allumage au contact. La transition est sur la sortie seulement :
     l'allumage doit etre instantane, c'est un accuse de reception, alors
     que l'extinction peut s'adoucir. */
  .heurtee {
    transition: none;
  }

  /* La pastille de notification, posee dans le coin du bouton de chat. */
  .puce {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    border: 2px solid #0a0c0f;
  }
</style>

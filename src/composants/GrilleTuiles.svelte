<!--
  La grille de tuiles reordonnables, et le geste qui la range.

  **Un composant plutot qu'une copie par ecran.** Jouer et Editer portent
  chacun trois tuiles qui se rangent exactement pareil. Laisser le geste
  dans les deux ecrans reviendrait a en tenir deux copies, et deux copies
  finissent toujours par diverger le jour ou l'une des deux est corrigee.
  Ici, l'ecran dit quelles tuiles il a et comment les dessiner, la grille
  dit comment on les range.

  **Pourquoi des evenements de pointeur et pas le glisser de HTML5.**
  Le glisser natif ne part pas dans la fenetre de Cairn sous Windows.
  Tauri ouvre sa fenetre avec `dragDropEnabled`, ce qui installe la cible
  de depot de l'explorateur sur toute la surface de WebView2 ; cette
  cible avale le geste avant que la page le voie, et `dragstart` n'arrive
  jamais. Le reglage est desormais coupe dans la configuration, mais on
  ne revient pas au glisser natif pour autant : il ne sait pas montrer la
  grille se reorganiser sous le curseur, seulement un trait entre deux
  tuiles, et il ne marche ni au doigt ni au stylet. Le pointeur, lui,
  fait les trois.

  Le geste, tel que la maquette l'avait regle et que Flow l'a valide :
  on appuie, et soit on maintient un quart de seconde, soit on bouge de
  cinq pixels. La tuile se souleve, sa case devient creuse, une etiquette
  suit le curseur, et les autres tuiles se rangent en direct. Le lacher
  ne fait que garder ce qui est deja affiche.

  **Deux chemins, une seule regle.** La souris porte, le clavier pousse,
  et les deux demandent le nouvel ordre au meme calcul en Rust. Le
  clavier n'est pas une politesse : quelqu'un qui ne peut pas tenir un
  bouton en deplacant la souris doit pouvoir ranger son accueil, et ceux
  qui le peuvent trouvent souvent ca penible.

  Le raccourci est Ctrl et une fleche, et pas Espace pour saisir puis les
  fleches : sur une tuile, Espace et Entree l'ouvrent deja, et leur
  donner un second sens selon un mode invisible est le genre de chose
  qu'on s'explique une fois et qu'on ne retient jamais.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import { t } from '../lib/traduction.svelte';
  import { enregistrer, rangerTuiles, glisserTuiles } from '../lib/etat.svelte';
  import type { Grille } from '../lib/pont';
  import type { Tuile } from '../lib/constantes';

  let {
    grille,
    ordre,
    tuiles,
    accent,
    etirer = true,
    contenu,
  }: {
    /** Laquelle des deux grilles on range, voir `cairn_core::reglages`. */
    grille: Grille;
    /** L'ordre courant, tel que les reglages le portent. */
    ordre: string[];
    /** Le catalogue des tuiles de cette grille, par identifiant. */
    tuiles: Record<string, Tuile>;
    accent: string;
    /**
     * Toutes les tuiles prennent la hauteur de la plus haute.
     *
     * Vrai sur Jouer, ou les illustrations veulent toute la hauteur et
     * ou une rangee inegale ferait desordre. Faux sur Editer : sans
     * planche a montrer, une tuile etiree par la fiche ouverte de sa
     * voisine n'est plus qu'un grand rectangle vide.
     */
    etirer?: boolean;
    /**
     * Ce que l'ecran dessine dans une case, quand elle n'est pas creuse.
     *
     * Le troisieme parametre est a appeler au moment du clic, et il dit
     * si ce clic doit etre ignore. C'est une fonction et pas une valeur
     * pour une raison precise : la valeur serait celle du dernier rendu,
     * alors que ce qu'on veut savoir est l'etat au moment du clic.
     */
    contenu: import('svelte').Snippet<[string, Tuile, () => boolean]>;
  } = $props();

  /** Le temps de maintien qui souleve une tuile sans bouger la souris. */
  const MAINTIEN = 250;
  /** Le deplacement qui la souleve tout de suite, sans attendre. */
  const SEUIL = 5;

  /** La tuile soulevee, et la position de l'etiquette qui suit le curseur. */
  let portee = $state<string | null>(null);
  let suivi = $state<{ x: number; y: number } | null>(null);
  /**
   * La hauteur de la case au moment de la saisie.
   *
   * La case creuse la reprend telle quelle. Sans ca, soulever une tuile
   * dont la fiche est ouverte fait retomber toute la rangee d'une
   * centaine de pixels, et les deux autres sautent sous le curseur au
   * milieu du geste.
   */
  let hauteurCase = $state(0);

  const tuilePortee = $derived(portee ? tuiles[portee] : undefined);

  /** La grille, qui ne bouge jamais : c'est elle qui capte le pointeur. */
  let racine: HTMLDivElement | undefined = $state();

  // L'etat du geste en cours. Rien de tout ca n'est affiche, donc rien de
  // tout ca n'a besoin d'etre reactif.
  let minuterie: ReturnType<typeof setTimeout> | undefined;
  let candidate: string | null = null;
  let pointeur: number | null = null;
  let departX = 0;
  let departY = 0;
  let dernierX = 0;
  let dernierY = 0;
  let bouge = false;
  /** Un reordonnancement est parti vers Rust et n'est pas revenu. */
  let enVol = false;
  /** L'ordre a change pendant ce geste, il y a donc quelque chose a garder. */
  let range = false;
  /**
   * Le clic qui suit un lacher ne doit pas ouvrir la tuile.
   *
   * La capture du pointeur suffit en theorie, elle renvoie le clic sur la
   * grille et pas sur la tuile. Ce drapeau est la ceinture : si la
   * capture echoue, par exemple parce que le navigateur l'a rendue plus
   * tot que prevu, le clic arrive quand meme et il faut l'ignorer.
   */
  let clicEtouffe = false;
  const etouffe = () => clicEtouffe;

  function saisir(id: string) {
    const case_ = racine?.querySelector<HTMLElement>(`[data-tuile="${id}"]`);
    hauteurCase = case_ ? Math.round(case_.getBoundingClientRect().height) : 0;
    portee = id;
    range = false;
    suivi = { x: dernierX, y: dernierY };
    // La capture garde le geste vivant quand le curseur sort de la
    // fenetre, et renvoie le clic final sur la grille. Elle est posee sur
    // la grille et pas sur la case : Svelte deplace les cases quand
    // l'ordre change, et une capture posee sur un noeud qui bouge dans
    // l'arbre n'est pas garantie de survivre au deplacement.
    if (pointeur !== null) racine?.setPointerCapture(pointeur);
  }

  function auPointeurEnfonce(e: PointerEvent, id: string) {
    clicEtouffe = false;
    if (e.button !== 0) return;
    // Les boutons poses dans une tuile, le point d'interrogation comme
    // ceux d'une fiche ouverte : appuyer dessus n'est pas prendre la tuile.
    if ((e.target as HTMLElement | null)?.closest('button')) return;
    candidate = id;
    pointeur = e.pointerId;
    departX = dernierX = e.clientX;
    departY = dernierY = e.clientY;
    bouge = false;
    clearTimeout(minuterie);
    minuterie = setTimeout(() => saisir(id), MAINTIEN);
  }

  function auPointeurBouge(e: PointerEvent) {
    if (candidate === null && portee === null) return;
    dernierX = e.clientX;
    dernierY = e.clientY;
    if (
      candidate !== null &&
      !bouge &&
      Math.abs(e.clientX - departX) + Math.abs(e.clientY - departY) > SEUIL
    ) {
      bouge = true;
      if (portee === null) {
        clearTimeout(minuterie);
        saisir(candidate);
      }
    }
    if (portee === null) return;
    suivi = { x: e.clientX, y: e.clientY };
    survoler(e.clientX, e.clientY);
  }

  /**
   * La case sous le curseur, et le nouvel ordre si c'en est une autre.
   *
   * On interroge le document plutot que de comparer les rectangles des
   * cases a la main : c'est le navigateur qui sait ce qui est vraiment
   * sous le curseur, y compris quand deux choses se chevauchent.
   * L'etiquette qui suit le curseur ne gene pas, elle laisse passer les
   * evenements.
   */
  async function survoler(x: number, y: number) {
    if (enVol || portee === null) return;
    const dessous = document.elementFromPoint(x, y)?.closest<HTMLElement>('[data-tuile]');
    const cible = dessous?.dataset.tuile;
    if (!cible || cible === portee) return;
    // Une case d'une autre grille ne compte pas. Le cas ne se presente
    // pas aujourd'hui, un seul ecran est affiche a la fois, mais la
    // garde coute une ligne et evite une surprise le jour ou deux
    // grilles cohabiteraient.
    if (!racine?.contains(dessous ?? null)) return;
    enVol = true;
    try {
      if (await glisserTuiles(grille, portee, cible)) range = true;
    } finally {
      enVol = false;
    }
  }

  function auPointeurRelache() {
    clearTimeout(minuterie);
    candidate = null;
    bouge = false;
    if (portee !== null) {
      clicEtouffe = true;
      portee = null;
      suivi = null;
      // Un seul passage sur le disque par geste, et seulement si l'ordre
      // a vraiment change.
      if (range) enregistrer();
      range = false;
    }
    if (pointeur !== null && racine?.hasPointerCapture(pointeur)) {
      racine.releasePointerCapture(pointeur);
    }
    pointeur = null;
  }

  function auClavierTuile(e: KeyboardEvent, id: string) {
    // Le clavier reprend la main, donc le clic n'est plus a etouffer.
    // Sans cette ligne, Entree sur une tuile ne fait rien apres un
    // glisser, jusqu'au prochain appui de souris.
    clicEtouffe = false;
    if (!e.ctrlKey || e.altKey || e.metaKey) return;
    const pas = e.key === 'ArrowLeft' ? -1 : e.key === 'ArrowRight' ? 1 : 0;
    if (pas === 0) return;
    e.preventDefault();
    // La tuile deplacee garde le focus : sans ca, on pousse une fois, le
    // focus tombe sur le corps de la page, et la poussee suivante ne fait
    // rien. La case suit sa tuile quand Svelte reordonne la liste, donc
    // c'est bien la bonne qu'on refocalise.
    const case_ = e.currentTarget as HTMLElement;
    rangerTuiles(grille, { id, pas }).then(() =>
      case_.querySelector<HTMLElement>('.tile')?.focus(),
    );
  }
</script>

<svelte:window
  onpointermove={auPointeurBouge}
  onpointerup={auPointeurRelache}
  onpointercancel={auPointeurRelache}
/>

<div
  class="grille"
  class:enGlisse={portee !== null}
  class:libre={!etirer}
  bind:this={racine}
>
  {#each ordre as id (id)}
    {@const tuile = tuiles[id]}
    {#if tuile}
      <!-- Le glisser vit sur cette case et non dans la tuile : la tuile
           dit a quoi elle ressemble et ce qu'elle ouvre, la grille dit
           comment on la range. -->
      <div
        class="case"
        data-tuile={id}
        role="presentation"
        onpointerdown={(e) => auPointeurEnfonce(e, id)}
        onkeydown={(e) => auClavierTuile(e, id)}
      >
        {#if portee === id}
          <div class="creux" style="min-height:{hauteurCase}px;" aria-hidden="true">
            <div class="rond">+</div>
          </div>
        {:else}
          {@render contenu(id, tuile, etouffe)}
        {/if}
      </div>
    {/if}
  {/each}
</div>

<!-- L'etiquette qui suit le curseur. Elle est posee en dehors de la
     grille et en position fixe : dans la grille, le defilement de l'ecran
     la decalerait, et un debordement cache la couperait. -->
{#if tuilePortee && suivi}
  <div class="porteur" style="left:{suivi.x}px; top:{suivi.y}px;" aria-hidden="true">
    <Icone
      nom={tuilePortee.icone}
      taille={16}
      couleur={tuilePortee.couleur ?? accent}
      epaisseur={1.7}
    />
    <span>{t(tuilePortee.label)}</span>
  </div>
{/if}

<style>
  .grille {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--gouttiere, 16px);
    align-items: stretch;
  }
  .grille.libre {
    align-items: start;
  }
  /* Pendant le geste, le doigt et le stylet ne doivent pas faire defiler
     l'ecran, et le curseur dit qu'il tient quelque chose. Les deux ne
     valent que pendant le geste : une grille qui refuse le defilement en
     permanence serait une grille qu'on ne peut pas depasser au doigt. */
  .grille.enGlisse {
    touch-action: none;
    cursor: grabbing;
  }
  /* La case est une grille a une case pour que la tuile remplisse toute
     la hauteur de la rangee, comme quand elle etait elle meme l'element
     de la grille. */
  .case {
    display: grid;
    position: relative;
  }
  /* La place laissee libre par la tuile soulevee. Elle garde la hauteur
     mesuree a la saisie, pour que rien ne bouge sous le curseur. */
  .creux {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 15px;
    padding: var(--tuile-marge, 20px);
    min-height: var(--tuile-hauteur, 178px);
    background: rgba(255, 255, 255, 0.015);
    border: 1px dashed rgba(255, 255, 255, 0.16);
    box-shadow: inset 0 2px 14px rgba(0, 0, 0, 0.5);
  }
  .rond {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 1px dashed rgba(255, 255, 255, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.28);
    font-size: 17px;
    line-height: 1;
  }
  /* L'etiquette portee. Legerement penchee : c'est ce qui la detache de
     la grille tout de suite, sans avoir besoin d'une ombre plus lourde. */
  .porteur {
    position: fixed;
    z-index: 60;
    pointer-events: none;
    transform: translate(-50%, -50%) rotate(-1.5deg);
    background: #171b21;
    border: 1px solid rgba(255, 255, 255, 0.22);
    border-radius: 12px;
    padding: 11px 15px;
    display: flex;
    align-items: center;
    gap: 10px;
    box-shadow: 0 18px 42px rgba(0, 0, 0, 0.6);
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
  }
</style>

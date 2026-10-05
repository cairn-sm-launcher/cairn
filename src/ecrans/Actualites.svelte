<!--
  La section What's new et son carrousel.

  Les deux slides sont posees dans la meme cellule de grille, celle qui
  n'est pas active restant simplement invisible. La rangee prend donc
  toujours la hauteur de la plus grande, a n'importe quelle largeur de
  fenetre, et les commandes du carrousel ne bougent jamais. Sans ca,
  mesure au lot 16 : 70 px d'ecart entre les deux slides sur une fenetre
  de 1100 px.

  **Depuis le lot 25 le contenu est reel.** Les deux cartes d'une vue
  portent deux articles du fil, le meme que l'ecran Communaute, verifie
  et garde en cache par Rust. Le texte de maquette qui expliquait ce
  qu'on pourrait mettre ici ne sert plus que lorsqu'il n'y a aucune
  actualite : une explication de ce qui viendra n'a plus lieu d'etre
  quand c'est arrive.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import { t, langueCourante } from '../lib/traduction.svelte';
  import { SLIDE_S, CATEGORIES } from '../lib/constantes';
  import { reglages, ui, naviguer } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';
  import type { Article } from '../lib/pont';

  const accent = $derived(reglages.accent);

  /**
   * Au plus six articles, soit trois vues de deux cartes.
   *
   * L'accueil n'est pas le fil : il donne envie d'aller le lire. Vingt
   * articles ici feraient un carrousel que personne n'attend jusqu'au
   * bout, et l'ecran Communaute existe pour ca.
   */
  const MAXIMUM = 6;

  let articles = $state<Article[]>([]);

  // On ne force pas le reseau : si le cache est bon il est servi tout de
  // suite, et l'accueil ne doit pas attendre une requete pour s'afficher.
  $effect(() => {
    const langue = reglages.langue;
    pont
      .actualites(false, langue, [], '')
      .then((vue) => (articles = vue.articles.slice(0, MAXIMUM)))
      .catch(() => (articles = []));
  });

  /** Les articles par deux, une paire par vue. */
  const paires = $derived.by(() => {
    const sortie: Article[][] = [];
    for (let i = 0; i < articles.length; i += 2) sortie.push(articles.slice(i, i + 2));
    return sortie;
  });

  const SLIDES = $derived(Math.max(1, paires.length));

  const couleur = (id: string) => CATEGORIES.find((c) => c.id === id)?.couleur ?? '#838C96';
  const libelle = (id: string) => CATEGORIES.find((c) => c.id === id)?.libelle ?? 'Other';

  /**
   * Une icone par categorie.
   *
   * La meme icone teintee de six couleurs ne dit pas de quoi il s'agit,
   * et la couleur est deja portee par l'etiquette juste en dessous. Deux
   * fois la meme information, zero fois la deuxieme.
   */
  const ICONES: Record<string, string> = {
    tournoi: 'flame',
    maps: 'grid',
    creation: 'tools',
    technique: 'sliders',
    entraide: 'people',
    autre: 'info',
  };
  const icone = (id: string) => ICONES[id] ?? 'info';

  const region = () => (langueCourante() === 'fr' ? 'fr-FR' : 'en-US');
  const quand = (secondes: number) =>
    new Date(secondes * 1000).toLocaleDateString(region(), { month: 'long', day: 'numeric' });

  // L'anneau se remplit sur la duree d'une vue. Avec Svelte le noeud ne
  // bouge plus d'un rendu a l'autre, donc l'animation n'est plus a
  // recaler par un delai negatif : elle repart quand on change de vue,
  // et c'est exactement ce qu'on veut.
  const PERIMETRE = 91.11;

  const aller = (i: number) => (ui.slide = i);
  const suivante = () => aller((ui.slide + 1) % SLIDES);
  const precedente = () => aller((ui.slide - 1 + SLIDES) % SLIDES);

  // Le nombre de vues depend maintenant du fil. Un fil plus court que la
  // derniere fois laisserait `ui.slide` au dela du compte, donc sur une
  // pile vide : on le ramene dans les bornes.
  $effect(() => {
    if (ui.slide >= SLIDES) ui.slide = 0;
  });

  // Le defilement automatique. Il s'arrete des qu'on pose le curseur sur
  // une carte : on ne fait pas disparaitre un texte sous les yeux de
  // celui qui est en train de le lire.
  $effect(() => {
    if (ui.slideEnPause || SLIDES < 2) return;
    const debut = ui.slide;
    const minuteur = setTimeout(() => {
      if (ui.slide === debut) suivante();
    }, SLIDE_S * 1000);
    return () => clearTimeout(minuteur);
  });
</script>

<div class="section">
  <div class="titre display">{t("What's new?")}</div>

  <div
    class="pile"
    role="presentation"
    onmouseenter={() => (ui.slideEnPause = true)}
    onmouseleave={() => (ui.slideEnPause = false)}
  >
    {#if paires.length === 0}
      <!-- Aucun article : on le dit et on montre ou ca vivra, plutot que
           de laisser une rangee vide a la place la plus visible. -->
      <div class="paire">
        <button class="carte vers" type="button" onclick={() => naviguer('community')}>
          <Icone nom="people" taille={19} couleur={accent} />
          <div class="carte-titre display">{t('Community News')}</div>
          <div class="corps large">
            {t('No news yet. They will show up here as soon as something is published.')}
          </div>
        </button>
      </div>
    {:else}
      {#each paires as paire, n (n)}
        <div class="paire" class:cachee={ui.slide !== n}>
          {#each paire as article (article.id)}
            <button
              class="carte vers"
              type="button"
              style="--teinte:{couleur(article.categorie)};"
              onclick={() => naviguer('community')}
            >
              <Icone nom={icone(article.categorie)} taille={19} couleur={couleur(article.categorie)} />
              <div class="haut">
                <span class="genre">{t(libelle(article.categorie))}</span>
                <span class="quand">{quand(article.date)}</span>
              </div>
              <div class="carte-titre display">{article.titre}</div>
              {#if article.resume}
                <div class="corps large">{article.resume}</div>
              {/if}
            </button>
          {/each}
          {#if paire.length === 1}
            <!-- Une paire impaire garde sa seconde place vide plutot que
                 d'etirer la premiere carte sur toute la largeur : les
                 cartes doivent faire la meme taille d'une vue a l'autre,
                 sinon la rangee saute a chaque changement. -->
            <div class="carte creuse" aria-hidden="true"></div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <div class="nav">
    <button class="arrow-btn" type="button" title={t('Previous slide')} onclick={precedente}>
      <Icone nom="chevL" taille={16} epaisseur={2} />
    </button>

    <div class="points">
      {#each Array.from({ length: SLIDES }, (_, i) => i) as n (n)}
        <button
          class="dot"
          type="button"
          title={t('Slide {0}', n + 1)}
          aria-label={t('Slide {0}', n + 1)}
          onclick={() => aller(n)}
        >
          <span
            class="barre"
            style="width:{ui.slide === n ? '18px' : '7px'};background:{ui.slide === n
              ? accent
              : 'rgba(255,255,255,0.20)'};"
          ></span>
        </button>
      {/each}
    </div>

    <button class="arrow-btn anneau" type="button" title={t('Next slide')} onclick={suivante}>
      <svg width="34" height="34" viewBox="0 0 34 34" aria-hidden="true">
        <circle cx="17" cy="17" r="14.5" fill="none" stroke="rgba(255,255,255,0.10)" stroke-width="1.6" />
        {#key ui.slide}
          <circle
            class="ring-run"
            cx="17"
            cy="17"
            r="14.5"
            fill="none"
            stroke={accent}
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-dasharray={PERIMETRE}
            style="animation-duration:{SLIDE_S}s;{ui.slideEnPause
              ? 'animation-play-state:paused;'
              : ''}"
          />
        {/key}
      </svg>
      <Icone nom="chev" taille={16} epaisseur={2} />
    </button>
  </div>
</div>

<style>
  .section {
    /* Le blanc entre les tuiles et les actualites suit la densite. */
    margin-top: var(--air-section, 28px);
  }
  .titre {
    font-weight: 700;
    font-size: 18px;
    letter-spacing: -0.02em;
    margin-bottom: 14px;
  }
  .pile {
    display: grid;
  }
  /* Une carte devient cliquable : elle mene au fil. C'est un bouton et
     pas une carte avec un lien dedans, pour que la cible soit la carte
     entiere et pas trois mots au milieu. */
  .vers {
    border: none;
    text-align: left;
    cursor: pointer;
    font: inherit;
    color: inherit;
    width: 100%;
  }
  .creuse {
    visibility: hidden;
  }
  .haut {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-top: 10px;
  }
  .genre {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--teinte, #838c96);
  }
  .quand {
    font-size: 11px;
    color: #6f7883;
  }
  .paire {
    grid-area: 1 / 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--gouttiere, 16px);
    align-items: stretch;
  }
  .cachee {
    visibility: hidden;
    pointer-events: none;
  }
  .carte {
    background: #13161b;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 15px;
    padding: 20px 22px;
    /* La maquette calibrait 300 px pour des listes a puces. Un article
       tient en trois lignes, donc la carte se retrouvait aux deux tiers
       vide. Le plancher descend a la hauteur d'un resume de deux lignes,
       et `align-items: stretch` sur la paire garde les deux cartes a la
       meme hauteur quelle que soit la plus longue. */
    min-height: 148px;
    display: flex;
    flex-direction: column;
  }
  .carte-titre {
    font-weight: 600;
    font-size: 15px;
    margin-top: 12px;
  }
  .vers .carte-titre {
    margin-top: 6px;
  }
  .corps {
    font-size: 12px;
    color: #9aa3ad;
    line-height: 1.55;
    margin-top: 8px;
  }
  .large {
    font-size: 13px;
    line-height: 1.6;
    margin-top: 10px;
  }
  .nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 16px;
  }
  .arrow-btn {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: transparent;
    color: #9aa3ad;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }
  .anneau {
    position: relative;
    border: none;
  }
  .anneau svg {
    position: absolute;
    left: 0;
    top: 0;
    transform: rotate(-90deg);
  }
  .points {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  /* Le bouton fait 26 par 16, la barre visible reste fine : la cible de
     clic est bien plus grande que ce que l'oeil voit. */
  .dot {
    width: 26px;
    height: 16px;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .barre {
    height: 6px;
    border-radius: 999px;
    display: inline-block;
    transition:
      width 0.2s ease,
      background 0.2s ease;
  }
</style>

<!--
  L'ecran Communaute : le fil d'actualites.

  **Sans serveur, sans compte, sans donnee personnelle.** Les articles
  sont des fichiers markdown dans un depot public, assembles en un JSON
  signe pose sur un hebergement statique. Cairn le telecharge, verifie la
  signature, le garde en cache et l'affiche. La moderation est la revue
  d'une proposition de modification sur le depot, avec son historique :
  c'est le dispositif le plus simple qui reponde a l'exigence de Flow.

  Ce qui n'est pas ici et qui vit en Rust, parce que ca merite des tests :
  la signature, le decoupage du markdown, le tri, les comptes et le
  filtre. Voir `cairn_core::actualites`. Cet ecran affiche.

  Un article se deplie sur place, comme une fiche serveur et comme une
  tuile d'Editer. Trois facons differentes d'ouvrir un detail dans la
  meme application, ce serait deux de trop.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Corps from '../composants/Corps.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import { t, langueCourante } from '../lib/traduction.svelte';
  import { reglages } from '../lib/etat.svelte';
  import { CATEGORIES } from '../lib/constantes';
  import * as pont from '../lib/pont';
  import type { CategorieActu, SouciActu, VueActualites } from '../lib/pont';

  const accent = $derived(reglages.accent);

  let choisies = $state<CategorieActu[]>([]);
  let recherche = $state('');
  let vue = $state<VueActualites | null>(null);
  let erreur = $state<string | null>(null);
  let chargement = $state(true);
  let rafraichit = $state(false);
  let ouvert = $state<string | null>(null);

  async function charger(forcer: boolean) {
    if (forcer) rafraichit = true;
    try {
      vue = await pont.actualites(forcer, reglages.langue, choisies, recherche);
      erreur = null;
    } catch (e) {
      erreur = String(e);
    } finally {
      chargement = false;
      rafraichit = false;
    }
  }

  // Le filtre, la recherche et la langue repartent tout de suite, sans
  // aller voir le reseau : le fil est deja la, c'est Rust qui le retrie.
  $effect(() => {
    void choisies;
    void recherche;
    void reglages.langue;
    charger(false);
  });

  const libelle = (id: string) => CATEGORIES.find((c) => c.id === id)?.libelle ?? 'Other';
  const couleur = (id: string) => CATEGORIES.find((c) => c.id === id)?.couleur ?? '#838C96';

  function basculer(id: CategorieActu) {
    choisies = choisies.includes(id) ? choisies.filter((c) => c !== id) : [...choisies, id];
  }

  /**
   * La phrase d'un souci, ecrite ici et pas en Rust.
   *
   * Rust rend une cle, l'interface la traduit : c'est la regle que le
   * projet s'etait donnee pour la date du profil et que le chemin des
   * erreurs avait oubliee. Le banc d'essai du lot 25 l'a revelee en
   * affichant une phrase francaise sur un ecran anglais.
   */
  function phrase(souci: SouciActu): string {
    switch (souci) {
      case 'reseau':
        return t('The news could not be fetched.');
      case 'signature':
        return t('The news could not be authenticated, so it was not used.');
      case 'illisible':
        return t('The news feed could not be read.');
      case 'rejeu':
        return t('The news received was older than the one already known, so it was ignored.');
      case 'tropgros':
        return t('The news feed is unusually large, so it was not used.');
      case 'nonconfigure':
        return t('No news feed is set up yet. See docs/actualites in the project folder.');
    }
  }

  const region = () => (langueCourante() === 'fr' ? 'fr-FR' : 'en-US');
  const quand = (secondes: number) =>
    new Date(secondes * 1000).toLocaleDateString(region(), {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
</script>

<div class="ecran">
  <header class="pad entete">
    <div>
      <h1 class="titre display">{t('Community')}</h1>
      <p class="sous-titre">
        {#if vue?.publie}
          {t('Updated {0}', quand(vue.publie))}
        {:else}
          {t('Announcements from the ShootMania community.')}
        {/if}
      </p>
    </div>
    <Bouton
      libelle={rafraichit ? t('Refreshing...') : t('Refresh')}
      genre="fantome"
      {accent}
      onclick={() => charger(true)}
    />
  </header>

  {#if vue?.avertissement}
    <!-- Des articles ET un avertissement : un cache valable avec un
         reseau coupe n'est pas une panne, c'est une information. On ne
         choisit pas entre montrer les actualites et dire pourquoi elles
         ne sont pas fraiches. -->
    <div class="pad">
      <p class="avertissement">
        <Icone nom="info" taille={15} couleur="#FFC857" epaisseur={1.7} />
        <span>
          {vue.articles.length > 0
            ? t('{0} Showing the last ones kept on this computer.', phrase(vue.avertissement))
            : phrase(vue.avertissement)}
        </span>
      </p>
    </div>
  {/if}

  {#if vue && vue.comptes.length > 0}
    <div class="pad barre">
      <label class="chercher">
        <Icone nom="search" taille={15} couleur="#838C96" epaisseur={1.8} />
        <input
          class="champ"
          type="search"
          bind:value={recherche}
          placeholder={t('Search the news')}
          aria-label={t('Search the news')}
        />
      </label>
      <!-- Aucune categorie cochee veut dire toutes, contrairement au
           filtre du journal ou tout decocher vide la liste. La difference
           tient a ce que les cases disent : la-bas elles sont cochees au
           depart et on en retire, ici elles sont vides et on en ajoute. -->
      <div class="categories" role="group" aria-label={t('Filter by category')}>
        {#each vue.comptes as [id, combien] (id)}
          <button
            class="categorie"
            type="button"
            aria-pressed={choisies.includes(id)}
            class:eteinte={choisies.length > 0 && !choisies.includes(id)}
            style="--teinte:{couleur(id)};"
            onclick={() => basculer(id)}
          >
            <span class="point"></span>
            <span>{t(libelle(id))}</span>
            <span class="combien mono">{combien}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="pad">
    <div class="liste">
      {#if chargement}
        <p class="vide">{t('Loading...')}</p>
      {:else if erreur}
        <p class="vide">{erreur}</p>
      {:else if !vue || vue.articles.length === 0}
        <p class="vide">
          {recherche.trim() !== '' || choisies.length > 0
            ? t('Nothing matches. Try another word, or clear a category.')
            : t('No news yet. They will show up here as soon as something is published.')}
        </p>
      {:else}
        {#each vue.articles as article (article.id)}
          <article class="carte" class:deplie={ouvert === article.id} style="--teinte:{couleur(article.categorie)};">
            <button
              class="tete"
              type="button"
              aria-expanded={ouvert === article.id}
              onclick={() => (ouvert = ouvert === article.id ? null : article.id)}
            >
              <span class="haut">
                <span class="etiquette">{t(libelle(article.categorie))}</span>
                {#if article.epingle}
                  <span class="epingle" title={t('Pinned')}>
                    <Icone nom="star" taille={12} couleur={couleur(article.categorie)} epaisseur={2} />
                  </span>
                {/if}
                <span class="date">{quand(article.date)}</span>
              </span>
              <span class="nom display">{article.titre}</span>
              {#if article.resume}
                <span class="resume">{article.resume}</span>
              {/if}
              <span class="chev" class:tourne={ouvert === article.id}>
                <Icone nom="chev" taille={16} couleur="#6F7883" epaisseur={2} />
              </span>
            </button>
            {#if ouvert === article.id}
              <div class="contenu">
                <Corps blocs={article.corps} />
              </div>
            {/if}
          </article>
      {/each}
      {/if}
    </div>
  </div>

  <p class="pad note">
    <Icone nom="lock" taille={15} couleur="#838C96" epaisseur={1.7} />
    <span>
      {t(
        'News comes from a public, signed file. No account, nothing sent about you, and it still works offline.',
      )}
    </span>
  </p>
</div>

<style>
  .ecran {
    flex: 1;
    overflow-y: auto;
    padding-top: var(--air-haut, 24px);
    padding-bottom: 30px;
    position: relative;
    z-index: 1;
  }
  .entete {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 16px;
  }
  .titre {
    font-size: 21px;
    font-weight: 600;
    margin: 0;
  }
  .sous-titre {
    font-size: 12px;
    color: #838c96;
    margin: 3px 0 0;
  }
  .avertissement {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin: 0 0 16px;
    padding: 10px 13px;
    border-radius: 10px;
    font-size: 12.5px;
    color: #d8c79a;
    background: rgba(255, 200, 87, 0.08);
    border: 1px solid rgba(255, 200, 87, 0.26);
  }

  .barre {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 16px;
  }
  .chercher {
    display: flex;
    align-items: center;
    gap: 9px;
    flex: 1 1 220px;
    max-width: 340px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9px;
    padding: 0 11px;
    height: 34px;
  }
  .champ {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: #f4f6f8;
    font-size: 12.5px;
  }
  .categories {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .categorie {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 12px;
    border-radius: 9px;
    border: 1px solid color-mix(in srgb, var(--teinte) 34%, transparent);
    background: color-mix(in srgb, var(--teinte) 11%, transparent);
    color: #e6e9ee;
    font-size: 12.5px;
    cursor: pointer;
  }
  .categorie.eteinte {
    border-color: rgba(255, 255, 255, 0.09);
    background: transparent;
    color: #6f7883;
  }
  .point {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--teinte);
  }
  .categorie.eteinte .point {
    background: #3a424c;
  }
  .combien {
    font-size: 11.5px;
    color: #9aa3ad;
  }
  .categorie.eteinte .combien {
    color: #5b636d;
  }

  .liste {
    display: flex;
    flex-direction: column;
    gap: 10px;
    /* Le plafond est porte par un bloc INTERIEUR au `.pad`, jamais par le
       `.pad` lui meme : celui-ci se centre par marges automatiques, donc
       lui donner une largeur plus petite le detache de la gouttiere et la
       liste cesse d'etre alignee sur son titre.

       C'est la deuxieme fois. L'etape 32 l'avait rencontre sur la table
       de l'Historique et l'avait ecrit au journal, et je l'ai refait ici
       deux lots plus tard. D'ou le controle d'alignement ajoute au banc :
       une lecon qui tient dans un commentaire se reoublie, une lecon qui
       tient dans un controle ne se reoublie plus. */
    max-width: 980px;
  }
  .carte {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.022);
    overflow: hidden;
    position: relative;
  }
  /* Le filet a la couleur de la categorie, comme le filet ambre des
     favoris dans la liste des serveurs et le filet du palier dans le
     classement. Meme vocabulaire, rien de neuf a apprendre. */
  .carte::before {
    content: '';
    position: absolute;
    left: 0;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 3px;
    background: var(--teinte);
    opacity: 0.85;
  }
  .carte.deplie {
    background: rgba(255, 255, 255, 0.04);
    border-color: color-mix(in srgb, var(--teinte) 28%, transparent);
  }
  .tete {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 4px 16px;
    width: 100%;
    padding: 14px 18px 14px 20px;
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }
  .haut {
    grid-column: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }
  .etiquette {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--teinte);
  }
  .epingle {
    display: flex;
  }
  .date {
    font-size: 11.5px;
    color: #6f7883;
  }
  .nom {
    grid-column: 1;
    font-size: 15px;
    font-weight: 600;
    color: #f4f6f8;
  }
  .resume {
    grid-column: 1;
    font-size: 12.5px;
    color: #9aa3ad;
    margin-top: 3px;
  }
  .chev {
    grid-column: 2;
    grid-row: 1 / span 3;
    display: flex;
    transition: transform 160ms ease;
  }
  .chev.tourne {
    transform: rotate(90deg);
  }
  .contenu {
    padding: 2px 18px 18px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    margin-top: 2px;
    padding-top: 14px;
  }
  .vide {
    font-size: 12.5px;
    color: #838c96;
    padding: 34px 2px;
    margin: 0;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: var(--air-section, 40px);
    font-size: 11.5px;
    color: #838c96;
  }
  @media (prefers-reduced-motion: reduce) {
    .chev {
      transition: none;
    }
  }
</style>

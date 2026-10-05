<!--
  Le corps d'un article, dessine a partir des blocs rendus par Rust.

  **Il n'y a pas une seule chaine HTML ici, et c'est le sujet.** Rust
  decoupe le markdown en blocs et en fragments, ce composant pose un
  noeud Svelte par morceau. Donc pas de `{@html}`, donc rien a assainir,
  donc rien a oublier d'assainir. Un article qui contient
  `<script>alert(1)</script>` s'affiche comme ce texte la, parce qu'il
  n'existe aucun chemin par lequel il pourrait devenir autre chose.

  Les liens ne naviguent pas dans la fenetre : ils partent au navigateur
  du systeme, et Rust revalide l'adresse avant de l'ouvrir.
-->
<script lang="ts">
  import { t } from '../lib/traduction.svelte';
  import { reglages } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';
  import type { Bloc, Fragment } from '../lib/pont';

  let { blocs }: { blocs: Bloc[] } = $props();

  const accent = $derived(reglages.accent);

  function ouvrir(lien: string) {
    pont.ouvrirLien(lien).catch(() => {
      // Rust a refuse l'adresse ou n'a pas pu l'ouvrir. Il l'a deja
      // ecrit dans le journal ; ici il n'y a rien d'utile a dire au
      // lecteur au milieu d'un article.
    });
  }
</script>

{#snippet morceaux(fragments: Fragment[])}
  {#each fragments as f, i (i)}
    {#if f.sorte === 'texte'}{f.texte}{:else if f.sorte === 'gras'}<strong>{f.texte}</strong
      >{:else if f.sorte === 'italique'}<em>{f.texte}</em>{:else if f.sorte === 'code'}<code
        class="enligne mono">{f.texte}</code
      >{:else if f.sorte === 'lien' || f.sorte === 'image'}<button
        class="lien"
        type="button"
        style="color:{accent};"
        title={f.lien}
        onclick={() => ouvrir(f.lien)}
        >{f.texte || f.lien}{#if f.sorte === 'image'}<span class="genre">{t('image')}</span
          >{/if}</button
      >{/if}
  {/each}
{/snippet}

<div class="corps">
  {#each blocs as bloc, i (i)}
    {#if bloc.sorte === 'titre'}
      {#if bloc.niveau === 1}
        <h3 class="display t1">{bloc.texte}</h3>
      {:else if bloc.niveau === 2}
        <h4 class="display t2">{bloc.texte}</h4>
      {:else}
        <h5 class="display t3">{bloc.texte}</h5>
      {/if}
    {:else if bloc.sorte === 'paragraphe'}
      <p>{@render morceaux(bloc.fragments)}</p>
    {:else if bloc.sorte === 'liste'}
      {#if bloc.ordonnee}
        <ol>
          {#each bloc.points as point, j (j)}
            <li>{@render morceaux(point)}</li>
          {/each}
        </ol>
      {:else}
        <ul>
          {#each bloc.points as point, j (j)}
            <li>{@render morceaux(point)}</li>
          {/each}
        </ul>
      {/if}
    {:else if bloc.sorte === 'citation'}
      <blockquote style="border-color:{accent};">{@render morceaux(bloc.fragments)}</blockquote>
    {:else if bloc.sorte === 'code'}
      <pre class="mono">{bloc.texte}</pre>
    {:else if bloc.sorte === 'separateur'}
      <hr />
    {/if}
  {/each}
</div>

<style>
  .corps {
    font-size: 13.5px;
    line-height: 1.65;
    color: #c2c8d0;
    /* Une colonne de texte se lit mal au dela d'une certaine largeur :
       l'oeil perd sa ligne en revenant a gauche. */
    max-width: 72ch;
  }
  .corps :global(p) {
    margin: 0 0 12px;
  }
  .t1 {
    font-size: 16px;
    font-weight: 600;
    color: #f4f6f8;
    margin: 20px 0 8px;
  }
  .t2 {
    font-size: 14.5px;
    font-weight: 600;
    color: #f4f6f8;
    margin: 18px 0 7px;
  }
  .t3 {
    font-size: 13.5px;
    font-weight: 600;
    color: #e6e9ee;
    margin: 16px 0 6px;
  }
  .corps > :first-child {
    margin-top: 0;
  }
  ul,
  ol {
    margin: 0 0 12px;
    padding-left: 20px;
  }
  li {
    margin-bottom: 4px;
  }
  blockquote {
    margin: 0 0 12px;
    padding: 2px 0 2px 13px;
    border-left: 2px solid;
    color: #9aa3ad;
  }
  pre {
    margin: 0 0 12px;
    padding: 11px 13px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.07);
    font-size: 12px;
    color: #c9d1d9;
    overflow-x: auto;
    white-space: pre;
  }
  .enligne {
    font-size: 12.5px;
    padding: 1px 5px;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.07);
    color: #e6e9ee;
  }
  hr {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    margin: 18px 0;
  }
  /* Un bouton et pas une ancre : il n'y a pas de navigation dans cette
     fenetre, l'adresse part au navigateur du systeme. Un `<a href>`
     promettrait un comportement qu'on ne lui donne pas, et offrirait un
     « ouvrir dans un nouvel onglet » qui ne mene nulle part. */
  .lien {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .genre {
    font-size: 10.5px;
    color: #838c96;
    margin-left: 5px;
    text-decoration: none;
    display: inline-block;
  }
</style>

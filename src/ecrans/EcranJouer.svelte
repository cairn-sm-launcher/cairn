<!--
  L'ecran Jouer : les trois tuiles, puis les actualites.

  La tuile Popular est la seule branchee sur de vraies donnees a ce
  stade. Elle prend le serveur le plus peuple de la liste que Rust rend
  deja triee, donc l'interface ne trie rien.

  Le geste qui range les tuiles n'est pas ici, il est dans
  `GrilleTuiles`, que cet ecran partage avec Editer.
-->
<script lang="ts">
  import Tuile from '../composants/Tuile.svelte';
  import GrilleTuiles from '../composants/GrilleTuiles.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Actualites from './Actualites.svelte';
  import { t } from '../lib/traduction.svelte';
  import { TUILES } from '../lib/constantes';
  import {
    reglages,
    ui,
    systeme,
    message,
    chargerServeurs,
    enregistrer,
    naviguer,
  } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';

  const accent = $derived(reglages.accent);

  /**
   * Le serveur le plus peuple, ou rien tant que la liste n'est pas la.
   *
   * La liste arrive deja triee par frequentation, donc c'est le premier
   * qui convient, mais pas n'importe lequel. Depuis le lot 19c-2 elle
   * contient aussi les serveurs a mot de passe, les lobbies de
   * matchmaking et les relais. Aucun des trois n'a sa place sur une
   * tuile dont la promesse est « un clic et tu joues » : le premier
   * demande un mot de passe qu'on n'a pas, le deuxieme met en file, le
   * troisieme duplique une partie qui se joue ailleurs. Ils restent
   * evidemment dans Parcourir les serveurs, ou ils sont marques.
   */
  const populaire = $derived(
    systeme.serveurs.find((s) => !s.prive && !s.lobby && !s.relais) ?? null,
  );

  const legendes = $derived({
    favorites:
      reglages.favoris.length > 0
        ? t('{0} servers saved', reglages.favoris.length)
        : t('No servers saved'),
    // Le nom du serveur est pose a part, avec ses couleurs : c'est une
    // chaine venue du jeu. Le reste de la legende est a nous.
    popular: populaire
      ? `, ${populaire.mode}, ${t('{0}/{1} players', populaire.joueurs, populaire.maximum)}`
      : systeme.serveursCharges
        ? t('No server online right now')
        : t('Loading...'),
    browse: t('View servers currently online'),
  });

  // Les details de la tuile Popular. Ils viennent tous de la fiche que
  // l'API rend deja, sauf la latence qu'elle ne donne pas : plutot qu'un
  // chiffre invente, la ligne affiche un tiret.
  const details = $derived(
    populaire
      ? // Les etiquettes sont traduites ici et pas a l'affichage, pour
        // qu'elles restent des chaines litterales dans le source : c'est
        // ce qui permet a verifier-traduction.mjs de les voir.
        [
          [t('Title pack'), populaire.titre],
          [t('Script'), populaire.mode],
          [t('Login'), populaire.login],
          [t('Ping'), populaire.latence === null ? '—' : `${populaire.latence} ms`],
          // « Spectators » et pas « Spectator slots » : l'API donne le
          // nombre d'observateurs presents, pas le nombre de places.
          [t('Spectators'), `${populaire.observateurs}`],
        ]
      : [],
  );

  function ouvrirTuile(id: string) {
    if (id === 'popular') ui.popularOuvert = !ui.popularOuvert;
    else if (id === 'browse') {
      // Parcourir montre tout : ouvrir avec le filtre des favoris encore
      // actif ferait croire a une liste vide.
      reglages.serveurs_favoris = false;
      enregistrer();
      naviguer('servers');
    } else if (reglages.favoris.length > 0) {
      // La tuile Favorites mene a la meme liste, filtree. Un ecran de
      // plus pour montrer les memes lignes avec les memes actions
      // n'apporterait rien, et il faudrait le tenir a jour deux fois.
      reglages.serveurs_favoris = true;
      enregistrer();
      naviguer('servers');
    } else message(t('No servers saved'));
  }

  async function lancer(spectate: boolean) {
    if (!populaire) return;
    if (!reglages.profil) {
      message(t('Pick a profile first'));
      return;
    }
    message(
      spectate
        ? t('Spectating {0}...', populaire.nom)
        : t('Joining {0}...', populaire.nom),
    );
    try {
      await pont.rejoindre(populaire.login, populaire.titre, reglages.profil, spectate);
    } catch (e) {
      message(String(e));
    }
  }

  // La liste sert a la tuile Popular, elle est donc demandee des l'ouverture
  // de l'ecran et pas au moment ou l'on va voir les serveurs.
  $effect(() => {
    if (!systeme.serveursCharges) chargerServeurs();
  });
</script>

<div class="pad ecran">
  <GrilleTuiles
    grille="jouer"
    ordre={reglages.ordre_tuiles}
    tuiles={TUILES}
    {accent}
  >
    {#snippet contenu(id, tuile, etouffe)}
      <Tuile
        {tuile}
        {accent}
        legende={legendes[id as keyof typeof legendes] ?? ''}
        legendeSegments={id === 'popular' && populaire ? populaire.nom_segments : undefined}
        legendeNue={id === 'popular' && populaire ? populaire.nom : undefined}
        onclick={() => {
          if (etouffe()) return;
          ouvrirTuile(id);
        }}
        aide={id === 'popular'
          ? {
              titre: t('Server details'),
              onclick: () => (ui.popularDetails = !ui.popularDetails),
            }
          : undefined}
      >
        {#snippet supplement()}
          {#if id === 'popular' && ui.popularDetails && populaire}
            <div class="details slide-in" role="presentation" onclick={(e) => e.stopPropagation()}>
              {#each details as [etiquette, valeur] (etiquette)}
                <div class="ligne">
                  <span class="cle">{etiquette}</span>
                  <span class="valeur mono">{valeur}</span>
                </div>
              {/each}
            </div>
          {/if}
          {#if id === 'popular' && ui.popularOuvert && populaire}
            <div class="actions slide-in" role="presentation" onclick={(e) => e.stopPropagation()}>
              <Bouton libelle={t('Join')} {accent} onclick={() => lancer(false)} />
              <Bouton libelle={t('Spectate')} genre="fantome" {accent} onclick={() => lancer(true)} />
            </div>
          {/if}
        {/snippet}
      </Tuile>
    {/snippet}
  </GrilleTuiles>

  <Actualites />
</div>

<style>
  .ecran {
    flex: 1;
    overflow-y: auto;
    /* Le blanc sous le bandeau suit la densite, voir app.css. */
    padding-top: var(--air-haut, 24px);
    padding-bottom: 30px;
    position: relative;
    z-index: 1;
  }
  /* Le panneau est plus opaque que dans la maquette. Il s'ouvre par
     dessus l'illustration, et un fond a 3 % laissait les valeurs longues
     (un login, un title pack) se poser sur la partie claire du dessin. */
  .details {
    margin-top: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(13, 16, 20, 0.86);
    border-radius: 10px;
    padding: 12px 14px;
  }
  .ligne {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 11.5px;
    padding: 3px 0;
  }
  .cle {
    color: #838c96;
  }
  .valeur {
    color: #c9d1d9;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    gap: 9px;
    margin-top: 14px;
  }
  /* Dans une tuile les deux boutons se partagent la largeur, contrairement
     a ceux de l'assistant qui gardent leur taille propre. */
  .actions :global(button) {
    flex: 1;
    padding: 10px 12px;
    font-size: 12.5px;
    border-radius: 9px;
  }
</style>

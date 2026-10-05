<!--
  La coquille de Cairn.

  Elle tient trois choses : l'ambiance de fond, le choix entre l'assistant
  de premier lancement et l'application, et la navigation clavier.

  Ce qui a disparu en venant de la maquette : les quatre-vingts lignes de
  render() et patch() qui reconstruisaient la page entiere a chaque
  changement puis la recollaient noeud par noeud, en rattrapant au
  passage le focus et la position du curseur dans les champs. C'est le
  travail de Svelte, et c'est le vrai gain du portage.
-->
<script lang="ts">
  import Bandeau from './composants/Bandeau.svelte';
  import SelecteurProfil from './composants/SelecteurProfil.svelte';
  import Fermeture from './composants/Fermeture.svelte';
  import Assistant from './ecrans/Assistant.svelte';
  import EcranJouer from './ecrans/EcranJouer.svelte';
  import Reglages from './ecrans/Reglages.svelte';
  import EcranServeurs from './ecrans/EcranServeurs.svelte';
  import EcranEditer from './ecrans/EcranEditer.svelte';
  import EcranClassement from './ecrans/EcranClassement.svelte';
  import EcranHistorique from './ecrans/EcranHistorique.svelte';
  import EcranDeveloppeur from './ecrans/EcranDeveloppeur.svelte';
  import EcranCommunaute from './ecrans/EcranCommunaute.svelte';
  import AVenir from './ecrans/AVenir.svelte';
  import { t } from './lib/traduction.svelte';
  import {
    reglages,
    ui,
    systeme,
    variablesFond,
    demarrer,
    veiller,
    naviguer,
    reculer,
    avancer,
  } from './lib/etat.svelte';
  import { pontPresent } from './lib/pont';
  import { listen } from '@tauri-apps/api/event';
  import { ORDRE_ONGLETS as ONGLETS } from './lib/constantes';

  demarrer();

  // L'etat du jeu est la seule chose qui bouge sans qu'on agisse, et
  // c'est elle qui decide du chemin de lancement.
  //
  // **Ce que le lot 19d change.** Jusqu'ici cette boucle redemandait
  // l'etat COMPLET toutes les cinq secondes : registre, liste des
  // processus, dossier des profils, et deux lignes de journal a chaque
  // passage, pour une seule valeur qui bouge vraiment. Flow l'a releve,
  // et il a raison. Elle appelle maintenant `veille`, qui ne regarde que
  // ce qui change et ne redemande le releve complet que quand le dossier
  // des profils a bouge.
  //
  // Deux rythmes. Fenetre au premier plan : cinq secondes, comme avant,
  // la pastille suit. Fenetre en arriere plan : trente secondes. Le cas
  // qui compte est celui du second ecran, Cairn visible pendant qu'on
  // joue ; quelqu'un qui ne regarde pas l'application n'a pas besoin
  // qu'elle interroge le systeme douze fois par minute. Et le retour au
  // premier plan declenche un coup d'oeil immediat, donc rien n'est
  // jamais en retard au moment ou on regarde.
  // La croix de la fenetre. Rust empeche la fermeture et nous passe la
  // main : c'est nous qui posons la question, parce qu'une boite de
  // dialogue du systeme ne sait pas porter une case decochee ni refuser
  // le focus a son bouton dangereux.
  $effect(() => {
    if (!pontPresent()) return;
    let arreter: (() => void) | null = null;
    let annule = false;
    // Le `catch` est pose tout de suite et pas dans le nettoyage : une
    // promesse rejetee sans preneur ressort en erreur de console, et une
    // erreur de console fait echouer les bancs d'essai pour une raison
    // qui n'a rien a voir.
    listen('fermeture-demandee', () => {
      ui.fermetureDemandee = true;
    })
      .then((f) => {
        if (annule) f();
        else arreter = f;
      })
      .catch(() => {});
    return () => {
      annule = true;
      arreter?.();
    };
  });

  const RYTHME_ACTIF = 5000;
  const RYTHME_FOND = 30000;

  $effect(() => {
    if (!pontPresent()) return;
    let minuteur: ReturnType<typeof setInterval>;
    const regarder = () => {
      veiller().catch(() => {});
    };
    const cadencer = () => {
      clearInterval(minuteur);
      minuteur = setInterval(regarder, document.hasFocus() ? RYTHME_ACTIF : RYTHME_FOND);
    };
    const reprendre = () => {
      regarder();
      cadencer();
    };
    cadencer();
    window.addEventListener('focus', reprendre);
    window.addEventListener('blur', cadencer);
    return () => {
      clearInterval(minuteur);
      window.removeEventListener('focus', reprendre);
      window.removeEventListener('blur', cadencer);
    };
  });

  // Les raccourcis portent sur la **position** de la touche et pas sur
  // la lettre qu'elle produit : `event.code` et jamais `event.key`. La
  // touche etiquetee A chez Flow porte Q chez un anglophone, c'est la
  // meme position, donc le meme raccourci, sans table de dispositions a
  // tenir a jour.
  //
  // Ils sont ignores des qu'on saisit du texte ou qu'une surimpression
  // est ouverte : rien de plus desagreable qu'un onglet qui change parce
  // qu'on a tape une lettre dans un champ.
  // La navigation ne boucle pas : une liste qui boucle casse la
  // representation spatiale que l'utilisateur se fait du menu, il ne sait
  // plus ou il est. Mais une touche qui ne fait rien passe pour une
  // panne, d'ou l'accuse de reception : la rangee d'onglets fait un
  // rebond amorti de quelques pixels quand on bute.
  function buter(cote: 'gauche' | 'droite') {
    ui.rebondCote = cote;
    ui.rebond += 1;
  }

  function auClavier(e: KeyboardEvent) {
    if (ui.assistant || ui.captureTouche) return;

    // Alt et les fleches, avant tout le reste : c'est la convention du
    // navigateur, et c'est la seule facon de revenir en arriere pour qui
    // n'a pas de boutons lateraux sur sa souris. Elles gardent leur sens
    // dans un champ de saisie, contrairement aux deux touches de
    // navigation, parce qu'elles ne produisent pas de lettre.
    if (e.altKey && !e.ctrlKey && !e.metaKey) {
      if (e.code === 'ArrowLeft') {
        e.preventDefault();
        reculer();
      } else if (e.code === 'ArrowRight') {
        e.preventDefault();
        avancer();
      }
      return;
    }

    if (ui.selecteurProfil) return;
    const cible = e.target as HTMLElement | null;
    if (cible && /^(INPUT|TEXTAREA|SELECT)$/.test(cible.tagName)) return;
    if (e.ctrlKey || e.altKey || e.metaKey) return;

    const pas =
      e.code === reglages.touche_precedent
        ? -1
        : e.code === reglages.touche_suivant
          ? 1
          : 0;
    if (pas === 0) return;
    deplacer(pas);
  }

  /**
   * Les boutons lateraux de la souris, Precedent et Suivant.
   *
   * **Ils ne font plus ce que font les deux touches.** Jusqu'au lot 19e
   * ils appelaient la meme fonction, donc ils parcouraient la rangee
   * d'onglets, et Flow l'a releve : c'etait un doublon sans interet.
   * Precedent et Suivant, sur une souris, veulent dire une seule chose
   * pour tout le monde, et c'est celle du navigateur. Ils remontent
   * maintenant le chemin reellement parcouru, Reglages et Parcourir les
   * serveurs compris, ce que les deux touches ne savent pas faire.
   *
   * La surimpression n'est pas exclue ici : un retour qui ferme ce qui
   * est ouvert par dessus est exactement ce qu'on attend. Voir
   * `reculer()`.
   */
  function auxBoutons(e: MouseEvent) {
    if (!reglages.souris_laterale) return;
    if (ui.assistant || ui.captureTouche) return;
    if (e.button !== 3 && e.button !== 4) return;
    e.preventDefault();
    if (e.button === 3) reculer();
    else avancer();
  }

  function deplacer(pas: -1 | 1) {
    const i = ONGLETS.indexOf(ui.ecran);
    if (i === -1) return;
    const j = i + pas;
    if (j < 0 || j >= ONGLETS.length) {
      buter(pas < 0 ? 'gauche' : 'droite');
      return;
    }
    naviguer(ONGLETS[j]);
  }
</script>

<svelte:window
  onkeydown={auClavier}
  onmouseup={auxBoutons}
  onauxclick={(e) => reglages.souris_laterale && e.button > 2 && e.preventDefault()}
/>

<div
  class="root"
  class:no-anim={!reglages.animations}
  data-densite={reglages.densite}
  data-ecran={ui.ecran}
  style={variablesFond()}
>
  <div class="waves">
    <span class="w1"></span><span class="w2"></span><span class="w3"></span>
  </div>
  <div class="glow"></div>

  {#if !ui.pret}
    <!-- Le temps d'un aller-retour avec Rust. Volontairement vide : un
         indicateur qui clignote pendant cinquante millisecondes fait plus
         de bruit que d'usage. -->
  {:else if ui.assistant}
    <Assistant />
  {:else}
    <!-- Le bandeau ne se cache plus sur aucun ecran. Parcourir les
         serveurs le masquait quand il etait un sous-ecran de Jouer, ce
         qui coupait la navigation le temps qu'on y etait. Il est devenu
         un onglet a l'etape 30. -->
    <Bandeau />
    {#if ui.ecran === 'play'}
      <EcranJouer />
    {:else if ui.ecran === 'servers'}
      <EcranServeurs />
    {:else if ui.ecran === 'leaderboard'}
      <EcranClassement />
    {:else if ui.ecran === 'history'}
      <EcranHistorique />
    {:else if ui.ecran === 'dev'}
      <EcranDeveloppeur />
    {:else if ui.ecran === 'appsettings'}
      <Reglages />
    {:else if ui.ecran === 'edit'}
      <EcranEditer />
    {:else if ui.ecran === 'chat'}
      <AVenir
        icone="chat"
        titre={t('Chat')}
        explication={t(
          'Lobby and community chat will live here. Public rooms only, with a report button and no private messages.',
        )}
      />
    {:else if ui.ecran === 'community'}
      <EcranCommunaute />
    {:else}
      <AVenir
        icone="info"
        titre={t('Nothing here yet')}
        explication={t(
          'This screen has not been built.',
        )}
      />
    {/if}
    {#if ui.selecteurProfil}
      <SelecteurProfil />
    {/if}
  {/if}

  {#if ui.fermetureDemandee}
    <Fermeture />
  {/if}

  {#if ui.message}
    <div class="toast">{ui.message}</div>
  {/if}

  <!-- Le moteur et l'interface ne sont pas de la meme version. Ce n'est
       pas une panne, l'application tourne : elle affiche simplement
       moins que ce qu'elle sait afficher, et sans ce message on cherche
       longtemps. Ambre et non rouge, pour dire exactement ca. -->
  {#if systeme.contratDepareille}
    <div class="depareille" role="status">
      {t('Cairn is running an older engine than this interface. Restart it so the Rust side rebuilds, otherwise some things will be missing.')}
    </div>
  {/if}

  {#if systeme.erreur}
    <div class="erreur" role="alert">{systeme.erreur}</div>
  {/if}
</div>

<style>
  .toast {
    position: absolute;
    left: 50%;
    bottom: 30px;
    transform: translateX(-50%);
    background: #181c22;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 11px;
    padding: 11px 20px;
    font-size: 12.5px;
    font-weight: 600;
    color: #f4f6f8;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    white-space: nowrap;
    animation: toast-in 0.2s ease;
    z-index: 35;
  }
  .depareille {
    position: absolute;
    left: 20px;
    bottom: 20px;
    z-index: 34;
    max-width: 440px;
    background: rgba(40, 34, 18, 0.94);
    border: 1px solid rgba(255, 200, 87, 0.42);
    border-radius: 11px;
    padding: 12px 16px;
    font-size: 12px;
    color: #ffe6b0;
    line-height: 1.5;
  }
  /* Les deux ne se chevauchent pas : une vraie panne monte d'un cran. */
  .depareille ~ .erreur {
    bottom: 84px;
  }
  .erreur {
    position: absolute;
    left: 20px;
    bottom: 20px;
    z-index: 34;
    max-width: 420px;
    background: rgba(40, 22, 22, 0.94);
    border: 1px solid rgba(255, 107, 74, 0.4);
    border-radius: 11px;
    padding: 12px 16px;
    font-size: 12px;
    color: #ffd8cd;
    line-height: 1.5;
  }
</style>

<!--
  Une rangee de reglage : un titre, une explication, une commande.

  Toutes les rangees partagent la meme grille, ce qui fait qu'on balaye
  la colonne des commandes du regard sans relire la structure a chaque
  ligne. C'est le meme raisonnement que pour la liste des serveurs.
-->
<script lang="ts">
  let {
    titre,
    explication = '',
    derniere = false,
    children,
  }: {
    titre: string;
    explication?: string;
    /** La derniere rangee d'un bloc n'a pas de trait sous elle. */
    derniere?: boolean;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<div class="rangee" class:derniere>
  <div class="texte">
    <div class="titre">{titre}</div>
    {#if explication}<div class="explication">{explication}</div>{/if}
  </div>
  <div class="commande">{@render children?.()}</div>
</div>

<style>
  .rangee {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 15px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .derniere {
    border-bottom: none;
  }
  .texte {
    min-width: 0;
  }
  .titre {
    font-size: 13.5px;
    font-weight: 600;
  }
  .explication {
    font-size: 12px;
    color: #838c96;
    margin-top: 2px;
    line-height: 1.5;
  }
  .commande {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 9px;
  }
</style>

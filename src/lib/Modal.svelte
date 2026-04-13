<script lang="ts">
  import type { ModalConfig } from './ModalBuilder';

  export let config: ModalConfig | null = null;
  export let close: () => void = () => {};

  let inputValue: string = '';

  $: if (config) {
    inputValue = '';
  }
</script>

{#if config}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-overlay" on:click|self={close}>
    <div class="modal">
      {#if config.title}
        <h3>{config.title}</h3>
      {/if}

      {#if config.text && !config.isHtml}
        <p>{config.text}</p>
      {:else if config.text && config.isHtml}
        <div>{@html config.text}</div>
      {/if}

      {#if config.hasInput}
        <input 
          type="text" 
          class="modal-input" 
          bind:value={inputValue} 
          placeholder={config.inputPlaceholder} 
        />
      {/if}

      {#if config.buttons.length > 0}
        <div class="modal-actions">
          {#each config.buttons as button}
            <button 
              class={button.primary ? 'primary' : ''} 
              on:click={() => button.action(close, config.hasInput ? inputValue : undefined)}
            >
              {button.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

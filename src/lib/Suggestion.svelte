<script lang="ts">
  import Label from "flowbite-svelte/Label.svelte";
  import Input from "flowbite-svelte/Input.svelte";
  import Textarea from "flowbite-svelte/Textarea.svelte";
  import { t } from 'svelte-i18n';
  import { invoke } from "@tauri-apps/api/core";
  import { ModalBuilder, type ModalConfig } from "./ModalBuilder";
  import Modal from "./Modal.svelte";

  let activeModal: ModalConfig | null = null;

  function closeModal() {
    activeModal = null;
  }

  export let isOpen = false;
  export let close = () => { isOpen = false; };

  function handleSubmit() {
    try {
      const title = (document.getElementById('suggestion') as HTMLInputElement).value;
      const message = (document.getElementById('message') as HTMLTextAreaElement).value;

      invoke('submit_suggestion', { name: title, message })
      activeModal = new ModalBuilder()
        .setTitle($t('suggestion.success-title'))
        .setText($t('suggestion.success-message'))
        .addButton($t('suggestion.close'), (close: () => any) => close())
        .build();
    } catch (error) {
      activeModal = new ModalBuilder()
        .setTitle($t('suggestion.error-title'))
        .setText($t('suggestion.error-message'))
        .addButton($t('suggestion.close'), (close: () => any) => close())
        .build();
      console.error("Error gathering suggestion data:", error);
    }
    close();
  }

  function handleOverlayClick() {
    close();
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" on:click|self={handleOverlayClick}>
    <div class="modal suggestion-modal">

      <h3 class="suggestion-header">{$t('suggestion.title')}</h3>

      <div class="suggestion-form">
        <div class="suggestion-field">
          <Label for="suggestion">{$t('suggestion.title')}</Label>
          <Input id="suggestion" maxlength={64} placeholder={$t('suggestion.title-placeholder')} />
        </div>

        <div class="suggestion-field">
          <label for="message">{$t('suggestion.message')}</label>
          <Textarea id="message" class="!w-full" placeholder={$t('suggestion.message-placeholder')} rows={5} />
        </div>
      </div>

      <div class="suggestion-actions">
        <button class="btn-suggestion-close" on:click={close}>{$t('suggestion.close')}</button>
        <button class="btn-suggestion-send" on:click={handleSubmit}>{$t('suggestion.submit')}</button>
      </div>

    </div>
  </div>
{/if}
<Modal config={activeModal} close={closeModal} />
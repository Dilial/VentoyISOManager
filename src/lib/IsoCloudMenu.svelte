<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy, onMount } from 'svelte';
  import { ModalBuilder, type ModalConfig } from "./ModalBuilder";
  import Modal from "./Modal.svelte";
  import Progressbar from "flowbite-svelte/Progressbar.svelte";
  
  let activeModal: ModalConfig | null = null;

  export let listIsos: () => void = () => {};

  function closeModal() {
    activeModal = null;
  }

  export let isOpen = false;
  export let close = () => { isOpen = false; };
  export let ISOsFolder = "";

  let uniqueDistros: string[] = [];
  let selectedDistro: string | null = null;
  let distroVersions: any[] = [];
  
  let searchQuery = '';
  let isLoading = false;
  let errorMsg = '';

  let isDownloading = false;
  let downloadProgress = 0;
  let currentFile = "";
  let unlistenProgress: (() => void) | undefined;

  $: filteredDistros = uniqueDistros.filter(d => d.toLowerCase().includes(searchQuery.toLowerCase()));
  $: filteredVersions = distroVersions.filter(v => v.version.toLowerCase().includes(searchQuery.toLowerCase()));

  $: if (isOpen && uniqueDistros.length === 0 && !isLoading && !isDownloading) {
    loadDistros();
  }

  onMount(async () => {
    unlistenProgress = await listen('download_progress', (event: any) => {
      const { downloaded, total } = event.payload;
      if (total > 0) {
        downloadProgress = Math.round((downloaded / total) * 100);
      }
    });
  });

  onDestroy(() => {
    if (unlistenProgress) unlistenProgress();
  });

  async function loadDistros() {
    isLoading = true; errorMsg = ''; selectedDistro = null;
    try { uniqueDistros = await invoke('fetch_unique_distros'); } 
    catch (err) { errorMsg = String(err); } 
    finally { isLoading = false; }
  }

  async function selectDistro(distro: string) {
    isLoading = true; errorMsg = ''; searchQuery = '';
    try {
      distroVersions = await invoke('fetch_versions_for_distro', { distro });
      selectedDistro = distro;
    } 
    catch (err) { errorMsg = String(err); } 
    finally { isLoading = false; }
  }

  function goBack() {
    selectedDistro = null;
    distroVersions = [];
    searchQuery = '';
    errorMsg = '';
  }

  async function handleDownload(iso: any) {
    let folder = ISOsFolder;

    if (!folder) {
      folder = await invoke<string>('select_folder');
      if (!folder) return; 
    }
    
    const separator = navigator.userAgent.includes('Win') ? '\\' : '/';
    const dest = `${folder}${separator}${iso.distro}_${iso.version}.iso`;

    try {
      currentFile = `${iso.distro} v${iso.version}`;
      isDownloading = true;
      downloadProgress = 0;
      
      await invoke('download_iso', { url: iso.download_url, dest });
      
      downloadProgress = 100;
      setTimeout(() => {
        isDownloading = false;
        listIsos();
        activeModal = new ModalBuilder()
          .setTitle("Iso Info")
          .setText(`La iso se ha descargado con éxito en:\n${dest}`)
          .addButton("Cerrar", (close) => close(), true)
          .build();
      }, 500);

    } catch (err) {
      isDownloading = false;
      activeModal = new ModalBuilder()
        .setTitle("Error")
        .setText(`Error descargando la iso:\n${err}`)
        .build();
    }
  }

  function handleOverlayClick() {
    if (!isDownloading) close();
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" on:click|self={handleOverlayClick}>
    <div class="modal" style="max-width: 650px; display: flex; flex-direction: column;">
      
      {#if isDownloading}
        <div style="text-align: center; padding: 2rem 1rem;">
          <h2 style="margin-top: 0">Descargando...</h2>
          <p style="margin-bottom: 1.5rem;">{currentFile}</p>
          
          <Progressbar
            progress={downloadProgress}
            size="h-5"
            color="green"
            labelInside
            animate
            tweenDuration={100}
          />
        </div>
        
      {:else}
        {#if !selectedDistro}
          <h3>☁️ Catálogo de SO Oficiales</h3>
        {:else}
          <h3 style="display:flex; align-items:center; gap: 8px">
            <button on:click={goBack} style="padding: 2px 8px; font-size:14px; background: transparent; color: #396cd8; border: none; cursor:pointer">← Atrás</button>
            Versiones de {selectedDistro}
          </h3>
        {/if}
        
        <input 
          type="text" class="modal-input" 
          placeholder={!selectedDistro ? "Buscar distribución..." : "Buscar versión (ej. 24.04)..."} 
          bind:value={searchQuery}
          style="margin-bottom: 1rem;"
        />

        <div class="iso-list-container" style="max-height: 50vh; overflow-y: auto; padding: 1rem 0;">
          {#if isLoading}
            <p class="modal-computing">Conectando con la base de datos...</p>
          {:else if errorMsg}
            <p class="hash-fail">{errorMsg}</p>
          {:else if !selectedDistro}
            {#if filteredDistros.length === 0} <p class="empty-state">Sin resultados.</p> {:else}
              <ul id="iso-list" style="margin: 0; padding: 0;">
                {#each filteredDistros as distroOption}
                  <li style="margin-bottom: 0.5rem; cursor: pointer; display: flex; justify-content: space-between;" on:click={() => selectDistro(distroOption)}>
                    <strong>{distroOption}</strong><span style="color:#666">Explorar →</span>
                  </li>
                {/each}
              </ul>
            {/if}
          {:else}
            {#if filteredVersions.length === 0} <p class="empty-state">Sin versiones aquí.</p> {:else}
              <ul id="iso-list" style="margin: 0; padding: 0;">
                {#each filteredVersions as iso}
                  <li style="margin-bottom: 0.5rem;">
                    <div style="flex-grow: 1;">
                      <span class="hash-label" style="font-size: 1.1em;">Versión {iso.version}</span>
                      {#if iso.is_lts} <span style="font-size: 0.8em; background-color: #2f855a; color: white; padding: 2px 4px; border-radius: 4px; margin-left:8px">LTS</span> {/if}
                    </div>
                    <button on:click={() => handleDownload(iso)} style="background-color: #2f855a">⬇ Descargar</button>
                  </li>
                {/each}
              </ul>
            {/if}
          {/if}
        </div>

        <div class="modal-actions" style="margin-top: 1rem;">
          <button on:click={close} style="background-color: transparent; border: 1px solid #ccc; color: inherit;">Cerrar Menú</button>
        </div>
      {/if}
      
    </div>
  </div>
{/if}
<Modal config={activeModal} close={closeModal} />
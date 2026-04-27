<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ModalBuilder, type ModalConfig } from "./lib/ModalBuilder";
  import Modal from "./lib/Modal.svelte";
  import IsoCloudMenu from "./lib/IsoCloudMenu.svelte";
  import Progressbar from "flowbite-svelte/Progressbar.svelte";
  import { readableBytes } from "./lib/utils";
  import { t } from 'svelte-i18n';

  interface FileInfo {
    name: string;
    path: string;
    full_path: string;
    is_dir: boolean;
  }

  let ISOsFolder: string = "";
  let ISOsParentFolder: string = "";
  let isos: FileInfo[] = [];
  let diskSpaceInfo: [number, number, number, number] | null = null;
  $: relativePath = ISOsFolder.replace(ISOsParentFolder, "");
  $: breadcrumbs = relativePath.split(/[/\\]/).filter((p) => p !== "");
  $: if (ISOsFolder) {
    invoke<[number, number, number, number] | null>("fetch_disk_space", {
      path: ISOsFolder,
    })
      .then((res) => (diskSpaceInfo = res))
      .catch((err) =>
        console.error("Error al obtener el espacio del disco:", err),
      );
  }
  $: diskColor = diskSpaceInfo
    ? ((diskSpaceInfo[3] >= 80
        ? "red"
        : diskSpaceInfo[3] >= 60
          ? "yellow"
          : "green") as "red" | "yellow" | "green")
    : ("green" as const);
  $: labelTextClass = diskColor === 'yellow' ? 'text-gray-800' : 'text-white';

  let activeModal: ModalConfig | null = null;
  let isCloudMenuOpen: boolean = false;
  function closeModal() {
    activeModal = null;
  }

  async function selectFolder() {
    try {
      const selectedPath = await invoke<string | null>("select_folder");
      if (selectedPath) {
        ISOsFolder = selectedPath;
        ISOsParentFolder = selectedPath;
        listIsos();
      }
    } catch (error) {
      console.error("Error al seleccionar carpeta:", error);
    }
  }

  async function navigate(newPath: string) {
    ISOsFolder = newPath;
    console.log(ISOsFolder);
    await listIsos();
  }

  async function goBack() {
    if (ISOsFolder === ISOsParentFolder) return;

    const isWindows = ISOsFolder.includes("\\");
    const separator = isWindows ? "\\" : "/";

    const parts = ISOsFolder.split(separator);
    parts.pop();

    const newPath = parts.join(separator);
    navigate(newPath || ISOsParentFolder);
  }

  async function listIsos() {
    try {
      isos = await invoke<FileInfo[]>("list_isos", { path: ISOsFolder });
      diskSpaceInfo = await invoke<[number, number, number, number] | null>("fetch_disk_space", {path: ISOsFolder});
    } catch (error) {
      console.error("Error al listar ISOs:", error);
    }
  }

  /*async function downloadIso() {
    console.log("downloadIso clicked");
  }*/

  /*async function updateIso(isoName: string) {
    console.log("updateIso clicked for:", isoName);
    activeModal = new ModalBuilder()
      .setText('<p class="modal-computing">Calculando hash...</p>')
      .setHtmlText('<p class="modal-computing">Calculando hash...</p>')
      .build();
    const hash = await invoke<string>("obtain_hash", { path: isoName });
    console.log(hash);
    activeModal = new ModalBuilder()
      .setText('<p class="modal-computing">Buscando hash en la base de datos...</p>')
      .setHtmlText('<p class="modal-computing">Buscando hash en la base de datos...</p>')
      .build();
    const isoInfo = await invoke<{distro: string, version: string} | null>("verify_hash", { hash: hash });
    if (isoInfo != null) {
      activeModal = new ModalBuilder()
        .setTitle("Iso Info")
        .setText(`La iso es de ${isoInfo?.distro} con la version ${isoInfo?.version}`)
        .addButton("Cerrar", (close) => close(), true)
        .build();
        return;
    }
    activeModal = new ModalBuilder()
      .setTitle("Iso Info")
      .setText(`ISO no encontrada en la base de datos`)
      .addButton("Cerrar", (close) => close(), true)
      .build();

  }*/

  async function deleteIso(path: string, isoName: string) {
    console.log("deleteIso clicked for:", path);
    try {
      activeModal = new ModalBuilder()
        .setTitle($t('iso-delete.title-delete'))
        //.setText("Estas seguro de que quieres eliminar la iso " + isoName)
        .setText($t('iso-delete.message-delete', {values: { isoName }}))
        .addButton($t('modal.cancel'), (close) => close())
        .addButton($t('modal.confirm'), async (close) => {
          await invoke("delete_iso", { path, isosFolder: ISOsFolder });
          await listIsos();
          close();
          activeModal = new ModalBuilder()
            .setTitle($t('iso-delete.title-deleted'))
            //.setText(`La ISO ${isoName} ha sido eliminada correctamente.`)
            .setText(`${$t('iso-delete.message-deleted', { values: { isoName } })}`)
            .addButton($t('modal.close'), (close) => close(), true)
            .build();
        })
        .build();
    } catch (error) {
      console.error("Error al eliminar la ISO:", error);
      activeModal = new ModalBuilder()
        .setTitle($t('iso-delete.title-error'))
        //.setText("Hubo un error al eliminar el archivo.")
        .setText($t('iso-delete.message-error', {values: { isoName }}))
        .addButton($t('modal.close'), (close) => close())
        .build();
    }
  }

  function startHash(isoName: string) {
    activeModal = new ModalBuilder()
      .setTitle($t('hash.title-verify'))
      .setText(
        $t('hash.message-verify'),
      )
      .addInput($t('hash.expected-hash-input'))
      .addButton($t('modal.close'), (close) => close())
      .addButton(
        $t('modal.calculate'),
        async (close, input) => {
          close();
          await computeAndShowHashResult(isoName, input || "");
        },
        true,
      )
      .build();
  }

  async function computeAndShowHashResult(
    isoName: string,
    expectedHash: string,
  ) {
    // Show computing modal (no buttons)
    activeModal = new ModalBuilder()
      .setText('<p class="modal-computing">'+$t('hash.calculating')+'</p>')
      .setHtmlText('<p class="modal-computing">'+$t('hash.calculating')+'</p>')
      .build();

    try {
      const hash = await invoke<string>("obtain_hash", { path: isoName });

      let resultHtml = `<h3>${$t('hash.hash-result')}</h3>`;

      if (expectedHash === "") {
        resultHtml += `<p class="hash-display">${hash}</p>`;
      } else {
        const match = expectedHash.toLowerCase() === hash.toLowerCase();
        if (match) {
          resultHtml += `<p class="hash-status hash-ok">${$t('hash.hash-match')}</p>
                          <p class="hash-display">${hash}</p>`;
        } else {
          resultHtml += `<p class="hash-status hash-fail">${$t('hash.hash-mismatch')}</p>
                          <p class="hash-label">${$t('hash.expected-hash')}:</p>
                          <p class="hash-display">${expectedHash}</p>
                          <p class="hash-label">${$t('hash.obtained-hash')}:</p>
                          <p class="hash-display">${hash}</p>`;
        }
      }

      activeModal = new ModalBuilder()
        .setHtmlText(resultHtml)
        .addButton($t('modal.close'), (close) => close(), true)
        .build();
    } catch (error) {
      console.error("Error obtaining hash:", error);
      activeModal = new ModalBuilder()
        .setTitle($t('hash.title-error'))
        .setText($t('hash.message-error'))
        .addButton($t('modal.close'), (close) => close())
        .build();
    }
  }

  async function verifyHash(isoName: string) {
    activeModal = new ModalBuilder()
      .setText('<p class="modal-computing">'+$t('hash.calculating')+'</p>')
      .setHtmlText('<p class="modal-computing">'+$t('hash.calculating')+'</p>')
      .build();
    const hash = await invoke<string>("obtain_hash", { path: isoName });
    console.log(hash);
    activeModal = new ModalBuilder()
      .setText(
        '<p class="modal-computing">'+$t('hash.search-database')+'</p>',
      )
      .setHtmlText(
        '<p class="modal-computing">'+$t('hash.search-database')+'</p>',
      )
      .build();
    const isoInfo = await invoke<{ distro: string; version: string } | null>(
      "verify_hash",
      { hash: hash },
    );
    if (isoInfo != null) {
      activeModal = new ModalBuilder()
        .setTitle($t('hash.iso-info'))
        .setText(
          $t('hash.iso-message', { values: { distro: isoInfo?.distro, version: isoInfo?.version } })
        )
        .addButton($t('modal.close'), (close) => close(), true)
        .build();
      return;
    }

    activeModal = new ModalBuilder()
      .setTitle($t('hash.iso-not-found-title'))
      .setText($t('hash.iso-not-found-message'))
      .addButton($t('modal.close'), (close) => close(), true)
      .build();
  }
</script>

<main class="container">
  <h1>Ventoy ISO Manager</h1>

  <div class="toolbar">
    <button on:click={selectFolder}>{$t('tools.select-folder')}</button>
    <button on:click={listIsos}>{$t('tools.list-isos')}</button>
  </div>

  <div class="toolbar">
    <button
      class="btn-download"
      on:click={() => (isCloudMenuOpen = true)}
      disabled={ISOsFolder == null || ISOsFolder == ""}
      style="background-color: #2f855a;">{$t('tools.download-iso')}</button
    >
  </div>
  <div class="toolbar"></div>
  {#if diskSpaceInfo}
    <div class="disk-space-container" style="width: 100%; margin: 15px 0;">
      <Progressbar
        progress={diskSpaceInfo[3]}
        size="h-4"
        color={diskColor}
        labelInside
        precision={1}
        classes={{ label: labelTextClass }}
      />

      <div
        style="display: flex; justify-content: space-between; font-size: 0.85em; color: gray; margin-top: 6px;"
      >
        <!-- Conversión de Bytes a Gigabytes (dividiendo entre 1024^3) -->
        <!--<span>Libre: {(diskSpaceInfo[2] / 1073741824).toFixed(2)} GB</span>
        <span>Total: {(diskSpaceInfo[0] / 1073741824).toFixed(2)} GB</span>-->
        <span>{$t('tools.free-up-space')} {readableBytes(diskSpaceInfo[2])}</span>
        <span>{$t('tools.total-space')} {readableBytes(diskSpaceInfo[0])}</span>
      </div>
    </div>
  {/if}

  <section class="iso-list-container">
    <h2>{$t('isos.title')}</h2>
    <ul id="iso-list">
      {#if ISOsFolder != "" && ISOsFolder != null}
        <div class="breadcrumb-nav">
          <button
            class="btn-back"
            on:click={goBack}
            disabled={ISOsFolder === ISOsParentFolder}
            title="Volver atras">{$t('isos.back')}</button
          >
          <div class="breadcrumbs">
            <span
              class="breadcrumbs-item"
              on:click={() => navigate(ISOsParentFolder)}>{$t('isos.parent-folder')}</span
            >
            {#each breadcrumbs as crumb, i}
              <span class="breadcrumb-separator">/</span>
              <span
                class="breadcrumb-item"
                on:click={() => {
                  const targetPath =
                    ISOsParentFolder +
                    "/" +
                    breadcrumbs.slice(0, i + 1).join("/");
                  navigate(targetPath);
                }}>{crumb}</span
              >
            {/each}
          </div>
        </div>
      {/if}
      {#if isos.length === 0}
        <li class="empty-state">
          {$t('isos.no-isos')}
        </li>
      {:else}
        {#each isos as isoName}
          {#if !isoName.is_dir}
            <li>
              <span>{isoName.name}</span>
              <div class="iso-actions">
                <!--<button
                  class="btn-update-iso"
                  on:click={() => updateIso(isoName.full_path)}
                  >Actualizar ISO</button
              >-->
                <button
                  class="btn-delete-iso"
                  on:click={() => deleteIso(isoName.full_path, isoName.name)}
                  >{$t('isos.actions.delete')}</button
                >
                <button
                  class="btn-obtain-iso-hash"
                  on:click={() => startHash(isoName.full_path)}
                  >{$t('isos.actions.obtain-hash')}</button
                >
                <button
                  class="btn-obtain-iso-info"
                  on:click={() => verifyHash(isoName.full_path)}
                  >{$t('isos.actions.verify-distro')}</button
                >
              </div>
            </li>
          {:else}
            <li
              class="directory-item"
              on:click={() => navigate(isoName.full_path)}
            >
              <span>{isoName.name}</span>
            </li>
          {/if}
        {/each}
      {/if}
    </ul>
  </section>
</main>

<Modal config={activeModal} close={closeModal} />
<IsoCloudMenu
  bind:isOpen={isCloudMenuOpen}
  close={() => (isCloudMenuOpen = false)}
  {ISOsFolder}
  {listIsos}
  {diskSpaceInfo}
/>

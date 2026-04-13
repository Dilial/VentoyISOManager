<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ModalBuilder, type ModalConfig } from "./lib/ModalBuilder";
  import Modal from "./lib/Modal.svelte";
  import IsoCloudMenu from "./lib/IsoCloudMenu.svelte";
  import Progressbar from "flowbite-svelte/Progressbar.svelte";

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
        .setTitle("Confirmar eliminación")
        .setText("Estas seguro de que quieres eliminar la iso " + isoName)
        .addButton("Cancelar", (close) => close())
        .addButton("Confirmar", async (close) => {
          await invoke("delete_iso", { path, isosFolder: ISOsFolder });
          await listIsos();
          close();
        })
        .build();
    } catch (error) {
      console.error("Error al eliminar la ISO:", error);
      alert("Error al eliminar el archivo: " + error);
    }
  }

  function startHash(isoName: string) {
    activeModal = new ModalBuilder()
      .setTitle("Verificar Hash")
      .setText(
        "Introduce el hash oficial para comparar. Déjalo en blanco para solo obtener el hash.",
      )
      .addInput("Hash esperado (opcional)")
      .addButton("Cancelar", (close) => close())
      .addButton(
        "Calcular",
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
      .setText('<p class="modal-computing">Calculando hash...</p>')
      .setHtmlText('<p class="modal-computing">Calculando hash...</p>')
      .build();

    try {
      const hash = await invoke<string>("obtain_hash", { path: isoName });

      let resultHtml = `<h3>Resultado del Hash</h3>`;

      if (expectedHash === "") {
        resultHtml += `<p class="hash-display">${hash}</p>`;
      } else {
        const match = expectedHash.toLowerCase() === hash.toLowerCase();
        if (match) {
          resultHtml += `<p class="hash-status hash-ok">Los hashes coinciden. La ISO es correcta.</p>
                          <p class="hash-display">${hash}</p>`;
        } else {
          resultHtml += `<p class="hash-status hash-fail">Los hashes NO coinciden.</p>
                          <p class="hash-label">Esperado:</p>
                          <p class="hash-display">${expectedHash}</p>
                          <p class="hash-label">Obtenido:</p>
                          <p class="hash-display">${hash}</p>`;
        }
      }

      activeModal = new ModalBuilder()
        .setHtmlText(resultHtml)
        .addButton("Cerrar", (close) => close(), true)
        .build();
    } catch (error) {
      console.error("Error obtaining hash:", error);
      activeModal = new ModalBuilder()
        .setTitle("Error")
        .setText("Hubo un error al calcular el hash.")
        .addButton("Cerrar", (close) => close())
        .build();
    }
  }

  async function verifyHash(isoName: string) {
    activeModal = new ModalBuilder()
      .setText('<p class="modal-computing">Calculando hash...</p>')
      .setHtmlText('<p class="modal-computing">Calculando hash...</p>')
      .build();
    const hash = await invoke<string>("obtain_hash", { path: isoName });
    console.log(hash);
    activeModal = new ModalBuilder()
      .setText(
        '<p class="modal-computing">Buscando hash en la base de datos...</p>',
      )
      .setHtmlText(
        '<p class="modal-computing">Buscando hash en la base de datos...</p>',
      )
      .build();
    const isoInfo = await invoke<{ distro: string; version: string } | null>(
      "verify_hash",
      { hash: hash },
    );
    if (isoInfo != null) {
      activeModal = new ModalBuilder()
        .setTitle("Iso Info")
        .setText(
          `La iso es de ${isoInfo?.distro} con la version ${isoInfo?.version}`,
        )
        .addButton("Cerrar", (close) => close(), true)
        .build();
      return;
    }

    activeModal = new ModalBuilder()
      .setTitle("Iso Info")
      .setText(`ISO no encontrada en la base de datos`)
      .addButton("Cerrar", (close) => close(), true)
      .build();
  }
</script>

<main class="container">
  <h1>Ventoy ISO Manager</h1>

  <div class="toolbar">
    <button on:click={selectFolder}>Seleccionar Carpeta</button>
    <button on:click={listIsos}>Listar ISOs</button>
  </div>

  <div class="toolbar">
    <button
      class="btn-download"
      on:click={() => (isCloudMenuOpen = true)}
      disabled={ISOsFolder == null || ISOsFolder == ""}
      style="background-color: #2f855a;">Descargar ISOs</button
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
        <span>Libre: {(diskSpaceInfo[2] / 1073741824).toFixed(2)} GB</span>
        <span>Total: {(diskSpaceInfo[0] / 1073741824).toFixed(2)} GB</span>
      </div>
    </div>
  {/if}

  <section class="iso-list-container">
    <h2>ISOs Descargadas</h2>
    <ul id="iso-list">
      {#if ISOsFolder != "" && ISOsFolder != null}
        <div class="breadcrumb-nav">
          <button
            class="btn-back"
            on:click={goBack}
            disabled={ISOsFolder === ISOsParentFolder}
            title="Volver atras">⇐</button
          >
          <div class="breadcrumbs">
            <span
              class="breadcrumbs-item"
              on:click={() => navigate(ISOsParentFolder)}>Base</span
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
          No hay ISOs descargadas. Usa 'Descargar' para agregar una.
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
                  >Eliminar ISO</button
                >
                <button
                  class="btn-obtain-iso-hash"
                  on:click={() => startHash(isoName.full_path)}
                  >Obtener hash ISO</button
                >
                <button
                  class="btn-obtain-iso-info"
                  on:click={() => verifyHash(isoName.full_path)}
                  >Obtener Distro</button
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
/>

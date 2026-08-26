/**
 * Envelopamento das APIs Tauri v2 com fallback resiliente para ambiente web/mock
 */
const TauriBridge = {
  isTauri() {
    return typeof window !== 'undefined' && !!window.__TAURI__;
  },

  async openDirectoryDialog(defaultTitle = 'Selecionar Pasta') {
    if (this.isTauri() && window.__TAURI__.dialog) {
      try {
        const selected = await window.__TAURI__.dialog.open({
          directory: true,
          multiple: false,
          title: defaultTitle,
        });
        return selected;
      } catch (err) {
        console.error('Erro ao abrir diálogo Tauri:', err);
      }
    }
    // Fallback prompt para desenvolvimento web ou teste direto
    return window.prompt(`Digite o caminho da pasta (${defaultTitle}):`);
  },

  async scanFolders(sourcePath, targetPath) {
    if (this.isTauri() && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke('scan_folders', {
        sourcePath,
        targetPath,
      });
    }
    throw new Error('Ambiente Tauri não detectado');
  },

  async startCopy(request) {
    if (this.isTauri() && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke('start_copy', { request });
    }
    throw new Error('Ambiente Tauri não detectado');
  },

  async cancelCopy() {
    if (this.isTauri() && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke('cancel_current_copy');
    }
  },

  async onCopyProgress(callback) {
    if (this.isTauri() && window.__TAURI__.event) {
      return await window.__TAURI__.event.listen('copy-progress', (event) => {
        callback(event.payload);
      });
    }
    return () => {};
  },

  async onScanProgress(callback) {
    if (this.isTauri() && window.__TAURI__.event) {
      return await window.__TAURI__.event.listen('scan-progress', (event) => {
        callback(event.payload);
      });
    }
    return () => {};
  },
};

window.TauriBridge = TauriBridge;

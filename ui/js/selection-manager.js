class SelectionManager {
  constructor() {
    this.selectedPaths = new Set();
  }

  initSelection(files) {
    this.selectedPaths.clear();
    for (const f of files) {
      if (f.selected) {
        this.selectedPaths.add(f.relPath);
      }
    }
  }

  isSelected(relPath) {
    return this.selectedPaths.has(relPath);
  }

  toggle(relPath) {
    if (this.selectedPaths.has(relPath)) {
      this.selectedPaths.delete(relPath);
    } else {
      this.selectedPaths.add(relPath);
    }
  }

  selectAll(files) {
    for (const f of files) {
      this.selectedPaths.add(f.relPath);
    }
  }

  selectNewerAndNew(files) {
    this.selectedPaths.clear();
    for (const f of files) {
      if (f.status === 'newerInSource' || f.status === 'onlyInSource' || f.status === 'heavyInSource') {
        this.selectedPaths.add(f.relPath);
      }
    }
  }

  clear() {
    this.selectedPaths.clear();
  }

  invert(files) {
    for (const f of files) {
      if (this.selectedPaths.has(f.relPath)) {
        this.selectedPaths.delete(f.relPath);
      } else {
        this.selectedPaths.add(f.relPath);
      }
    }
  }

  getSelectedCount() {
    return this.selectedPaths.size;
  }

  getSelectedBytes(allFiles) {
    let bytes = 0;
    for (const f of allFiles) {
      if (this.selectedPaths.has(f.relPath)) {
        bytes += f.srcSize || 0;
      }
    }
    return bytes;
  }

  getSelectedList() {
    return Array.from(this.selectedPaths);
  }
}

// Helpers de formatação
const FormatUtils = {
  formatBytes(bytes) {
    if (bytes === null || bytes === undefined) return '-';
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  },

  formatDate(timestampMillis) {
    if (!timestampMillis) return '-';
    const d = new Date(timestampMillis);
    return d.toLocaleString('pt-BR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  },

  getStatusLabel(status) {
    switch (status) {
      case 'onlyInSource':
        return { text: 'Novo na Origem', class: 'badge-only-source' };
      case 'onlyInTarget':
        return { text: 'Apenas Destino', class: 'badge-only-target' };
      case 'newerInSource':
        return { text: 'Mais Recente', class: 'badge-newer' };
      case 'olderInSource':
        return { text: 'Mais Antigo', class: 'badge-older' };
      case 'heavyInSource':
        return { text: 'Mais Pesado', class: 'badge-heavy' };
      case 'lightInSource':
        return { text: 'Mais Leve', class: 'badge-light' };
      case 'equal':
        return { text: 'Idêntico', class: 'badge-equal' };
      default:
        return { text: 'Diferente', class: 'badge-different' };
    }
  },
};

window.SelectionManager = SelectionManager;
window.FormatUtils = FormatUtils;

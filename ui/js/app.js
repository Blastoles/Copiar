document.addEventListener('DOMContentLoaded', () => {
  // Estado do aplicativo
  const state = {
    sourcePath: '',
    targetPath: '',
    scanResult: null,
    allFiles: [],
    filteredFiles: [],
    activeTab: 'all',
    searchQuery: '',
    isScanning: false,
    isCopying: false,
    scanProgress: {
      srcCount: 0,
      tgtCount: 0,
      compareCount: 0,
    }
  };

  const selection = new SelectionManager();

  // Elementos DOM
  const dom = {
    btnBrowseSource: document.getElementById('btn-browse-source'),
    btnBrowseTarget: document.getElementById('btn-browse-target'),
    inputSource: document.getElementById('input-source'),
    inputTarget: document.getElementById('input-target'),
    btnScan: document.getElementById('btn-scan'),

    metricsPanel: document.getElementById('metrics-panel'),
    metricTotal: document.getElementById('metric-total'),
    metricSizeTotal: document.getElementById('metric-size-total'),
    metricNewer: document.getElementById('metric-newer'),
    metricOnlySrc: document.getElementById('metric-only-src'),
    metricDiff: document.getElementById('metric-diff'),
    metricHeavy: document.getElementById('metric-heavy'),
    metricEqual: document.getElementById('metric-equal'),

    tabButtons: document.querySelectorAll('.tab-btn'),
    countTabAll: document.getElementById('count-tab-all'),
    countTabNewer: document.getElementById('count-tab-newer'),
    countTabOnlySource: document.getElementById('count-tab-only-source'),
    countTabDifferent: document.getElementById('count-tab-different'),
    countTabHeavy: document.getElementById('count-tab-heavy'),
    countTabEqual: document.getElementById('count-tab-equal'),

    searchInput: document.getElementById('search-input'),

    selectedCount: document.getElementById('selected-count'),
    filteredCount: document.getElementById('filtered-count'),
    selectedSize: document.getElementById('selected-size'),

    btnSelectNewer: document.getElementById('btn-select-newer'),
    btnSelectAllFiltered: document.getElementById('btn-select-all-filtered'),
    btnInvertSelection: document.getElementById('btn-invert-selection'),
    btnClearSelection: document.getElementById('btn-clear-selection'),

    masterCheckbox: document.getElementById('master-checkbox'),

    viewport: document.getElementById('virtual-scroll-viewport'),
    spacer: document.getElementById('virtual-scroll-spacer'),
    content: document.getElementById('virtual-scroll-content'),
    emptyState: document.getElementById('empty-state'),

    optPreserveMtime: document.getElementById('opt-preserve-mtime'),
    btnStartCopy: document.getElementById('btn-start-copy'),
    btnStartCopyText: document.getElementById('btn-start-copy-text'),

    // Modal
    copyModal: document.getElementById('copy-modal'),
    modalTitle: document.getElementById('modal-title'),
    modalBadgeSpeed: document.getElementById('modal-badge-speed'),
    modalCurrentFile: document.getElementById('modal-current-file'),
    modalFilesCount: document.getElementById('modal-files-count'),
    modalPercent: document.getElementById('modal-percent'),
    modalProgressBar: document.getElementById('modal-progress-bar'),
    modalBytesCount: document.getElementById('modal-bytes-count'),
    modalStatusText: document.getElementById('modal-status-text'),
    btnCancelCopy: document.getElementById('btn-cancel-copy'),
    btnCloseModal: document.getElementById('btn-close-modal'),
  };

  // Instanciar Virtual Table
  const virtualTable = new VirtualTable(dom.viewport, dom.spacer, dom.content, {
    rowHeight: 38,
    onSelectionChange: () => updateSelectionUI(),
  });

  // Event Listeners - Pastas
  dom.btnBrowseSource.addEventListener('click', async () => {
    const dir = await window.TauriBridge.openDirectoryDialog('Selecionar Pasta de Origem');
    if (dir) {
      state.sourcePath = dir;
      dom.inputSource.value = dir;
      checkCanScan();
    }
  });

  dom.btnBrowseTarget.addEventListener('click', async () => {
    const dir = await window.TauriBridge.openDirectoryDialog('Selecionar Pasta de Destino');
    if (dir) {
      state.targetPath = dir;
      dom.inputTarget.value = dir;
      checkCanScan();
    }
  });

  function checkCanScan() {
    dom.btnScan.disabled = !state.sourcePath || !state.targetPath;
  }

  // Event Listener - Comparar
  dom.btnScan.addEventListener('click', async () => {
    if (!state.sourcePath || !state.targetPath || state.isScanning) return;

    state.isScanning = true;
    dom.btnScan.disabled = true;
    state.scanProgress.srcCount = 0;
    state.scanProgress.tgtCount = 0;
    state.scanProgress.compareCount = 0;
    dom.btnScan.innerHTML = 'Comparando...';

    try {
      const result = await window.TauriBridge.scanFolders(state.sourcePath, state.targetPath);
      state.scanResult = result;
      state.allFiles = result.files || [];
      selection.initSelection(state.allFiles);

      updateMetricsUI(result.summary);
      applyFilterAndRender();
      dom.metricsPanel.classList.remove('hidden');
      dom.emptyState.classList.add('hidden');
    } catch (err) {
      alert(`Erro ao comparar pastas: ${err}`);
      console.error(err);
    } finally {
      state.isScanning = false;
      dom.btnScan.disabled = false;
      dom.btnScan.innerHTML = `
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        Comparar Pastas
      `;
    }
  });

  // Atualizar Métricas e Contadores das Abas
  function updateMetricsUI(summary) {
    dom.metricTotal.textContent = summary.totalItems;
    dom.metricSizeTotal.textContent = window.FormatUtils.formatBytes(summary.totalSrcBytes);
    dom.metricNewer.textContent = summary.newerCount;
    dom.metricOnlySrc.textContent = summary.onlySourceCount;
    dom.metricDiff.textContent = summary.differentCount;
    dom.metricHeavy.textContent = summary.heavyCount;
    dom.metricEqual.textContent = summary.equalCount;

    dom.countTabAll.textContent = summary.totalItems;
    dom.countTabNewer.textContent = summary.newerCount;
    dom.countTabOnlySource.textContent = summary.onlySourceCount;
    dom.countTabDifferent.textContent = summary.differentCount;
    dom.countTabHeavy.textContent = summary.heavyCount;
    dom.countTabEqual.textContent = summary.equalCount;
  }

  // Troca de Abas
  dom.tabButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      dom.tabButtons.forEach((b) => b.classList.remove('active'));
      btn.classList.add('active');
      state.activeTab = btn.getAttribute('data-tab');
      applyFilterAndRender();
    });
  });

  // Busca em Tempo Real
  dom.searchInput.addEventListener('input', (e) => {
    state.searchQuery = e.target.value.toLowerCase().trim();
    applyFilterAndRender();
  });

  function applyFilterAndRender() {
    let list = state.allFiles;

    // Filtro por Aba
    if (state.activeTab === 'newer') {
      list = list.filter((f) => f.status === 'newerInSource');
    } else if (state.activeTab === 'only-source') {
      list = list.filter((f) => f.status === 'onlyInSource');
    } else if (state.activeTab === 'different') {
      list = list.filter((f) => f.status !== 'equal' && f.status !== 'onlyInSource' && f.status !== 'onlyInTarget');
    } else if (state.activeTab === 'heavy') {
      list = list.filter((f) => f.srcSize !== null && f.targetSize !== null && f.sizeDiff > 0);
    } else if (state.activeTab === 'equal') {
      list = list.filter((f) => f.status === 'equal');
    }

    // Filtro por Busca
    if (state.searchQuery) {
      list = list.filter((f) => f.relPath.toLowerCase().includes(state.searchQuery));
    }

    state.filteredFiles = list;
    virtualTable.setItems(state.filteredFiles, selection);
    updateSelectionUI();
  }

  // Ações de Seleção em Lote
  dom.btnSelectNewer.addEventListener('click', () => {
    selection.selectNewerAndNew(state.filteredFiles);
    virtualTable.render();
    updateSelectionUI();
  });

  dom.btnSelectAllFiltered.addEventListener('click', () => {
    selection.selectAll(state.filteredFiles.filter((f) => f.status !== 'onlyInTarget'));
    virtualTable.render();
    updateSelectionUI();
  });

  dom.btnInvertSelection.addEventListener('click', () => {
    selection.invert(state.filteredFiles.filter((f) => f.status !== 'onlyInTarget'));
    virtualTable.render();
    updateSelectionUI();
  });

  dom.btnClearSelection.addEventListener('click', () => {
    selection.clear();
    virtualTable.render();
    updateSelectionUI();
  });

  dom.masterCheckbox.addEventListener('change', () => {
    if (dom.masterCheckbox.checked) {
      selection.selectAll(state.filteredFiles.filter((f) => f.status !== 'onlyInTarget'));
    } else {
      selection.clear();
    }
    virtualTable.render();
    updateSelectionUI();
  });

  function updateSelectionUI() {
    const selCount = selection.getSelectedCount();
    const selBytes = selection.getSelectedBytes(state.allFiles);

    dom.selectedCount.textContent = selCount;
    dom.filteredCount.textContent = state.filteredFiles.length;
    dom.selectedSize.textContent = window.FormatUtils.formatBytes(selBytes);

    dom.btnStartCopy.disabled = selCount === 0 || state.isCopying;
    dom.btnStartCopyText.textContent = `Iniciar ${getOpName()} (${selCount} arquivos - ${window.FormatUtils.formatBytes(selBytes)})`;
  }

  function getOpName() {
    const op = document.querySelector('input[name="op-type"]:checked')?.value || 'copy';
    return op === 'move' ? 'Movimentação' : 'Cópia';
  }

  document.querySelectorAll('input[name="op-type"]').forEach((radio) => {
    radio.addEventListener('change', () => updateSelectionUI());
  });

  // Operação de Cópia / Movimentação
  dom.btnStartCopy.addEventListener('click', async () => {
    const filesToCopy = selection.getSelectedList();
    if (filesToCopy.length === 0 || state.isCopying) return;

    const opType = document.querySelector('input[name="op-type"]:checked')?.value || 'copy';
    const preserveTimestamps = dom.optPreserveMtime.checked;

    state.isCopying = true;
    dom.copyModal.classList.remove('hidden');
    dom.modalTitle.textContent = opType === 'move' ? 'Movendo Arquivos...' : 'Copiando Arquivos...';
    dom.modalBadgeSpeed.textContent = '0.0 MB/s';
    dom.modalCurrentFile.textContent = 'Iniciando transferência...';
    dom.modalProgressBar.style.width = '0%';
    dom.modalPercent.textContent = '0%';
    dom.btnCancelCopy.classList.remove('hidden');
    dom.btnCloseModal.classList.add('hidden');

    const request = {
      sourceBase: state.sourcePath,
      targetBase: state.targetPath,
      filesToCopy,
      preserveTimestamps,
      operationType: opType,
    };

    try {
      const result = await window.TauriBridge.startCopy(request);
      if (result.errorCount > 0) {
        dom.modalStatusText.textContent = `Erro: ${result.errorCount} falhas. Detalhe: ${result.errors.join(', ')}`;
        dom.modalStatusText.style.color = '#ef4444';
        dom.modalTitle.textContent = 'Operação Concluída com Erros';
      } else {
        dom.modalStatusText.textContent = `Sucesso: ${result.successCount} arquivos processados.`;
        dom.modalStatusText.style.color = '';
      }
    } catch (err) {
      dom.modalStatusText.textContent = `Interrompido: ${err}`;
      dom.modalStatusText.style.color = '#ef4444';
    } finally {
      state.isCopying = false;
      dom.btnCancelCopy.classList.add('hidden');
      dom.btnCloseModal.classList.remove('hidden');
    }
  });

  // Escutar eventos de progresso do Tauri
  window.TauriBridge.onCopyProgress((progress) => {
    dom.modalCurrentFile.textContent = progress.currentFile;
    dom.modalFilesCount.textContent = `${progress.fileIndex} / ${progress.totalFiles} arquivos`;
    dom.modalPercent.textContent = `${Math.round(progress.percentageTotal)}%`;
    dom.modalProgressBar.style.width = `${progress.percentageTotal}%`;
    dom.modalBytesCount.textContent = `${window.FormatUtils.formatBytes(progress.totalBytesCopied)} / ${window.FormatUtils.formatBytes(progress.totalBytesToCopy)}`;
    dom.modalBadgeSpeed.textContent = `${(progress.speedBytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;

    if (progress.isFinished) {
      if (progress.hasError) {
        dom.modalTitle.textContent = 'Operação Concluída com Erros';
      } else {
        dom.modalTitle.textContent = 'Operação Concluída!';
      }
      dom.btnCancelCopy.classList.add('hidden');
      dom.btnCloseModal.classList.remove('hidden');
    }
  });

  window.TauriBridge.onScanProgress((progress) => {
    if (!state.isScanning) return;
    
    if (progress.phase === 'source') {
      state.scanProgress.srcCount = progress.count;
    } else if (progress.phase === 'target') {
      state.scanProgress.tgtCount = progress.count;
    } else if (progress.phase === 'comparison') {
      state.scanProgress.compareCount = progress.count;
    }

    if (state.scanProgress.compareCount > 0) {
      dom.btnScan.textContent = `Comparando... (${state.scanProgress.compareCount})`;
    } else {
      dom.btnScan.textContent = `Origem: ${state.scanProgress.srcCount} | Destino: ${state.scanProgress.tgtCount}`;
    }
  });

  dom.btnCancelCopy.addEventListener('click', async () => {
    await window.TauriBridge.cancelCopy();
    dom.modalStatusText.textContent = 'Cancelando...';
  });

  dom.btnCloseModal.addEventListener('click', () => {
    dom.copyModal.classList.add('hidden');
    // Re-executar scan para atualizar estado após a cópia
    dom.btnScan.click();
  });
});

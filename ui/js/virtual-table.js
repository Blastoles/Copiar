class VirtualTable {
  constructor(viewportEl, spacerEl, contentEl, options = {}) {
    this.viewport = viewportEl;
    this.spacer = spacerEl;
    this.content = contentEl;
    this.rowHeight = options.rowHeight || 38;
    this.overscan = options.overscan || 10;
    
    this.items = [];
    this.selectionManager = null;
    this.onSelectionChange = options.onSelectionChange || (() => {});

    this.viewport.addEventListener('scroll', () => this.render());
    window.addEventListener('resize', () => this.render());
  }

  setItems(items, selectionManager) {
    this.items = items;
    this.selectionManager = selectionManager;
    this.spacer.style.height = `${this.items.length * this.rowHeight}px`;
    this.viewport.scrollTop = 0;
    this.render();
  }

  render() {
    if (!this.items || this.items.length === 0) {
      this.content.innerHTML = '';
      this.spacer.style.height = '0px';
      return;
    }

    const scrollTop = this.viewport.scrollTop;
    const viewportHeight = this.viewport.clientHeight || 400;

    const startIndex = Math.max(0, Math.floor(scrollTop / this.rowHeight) - this.overscan);
    const endIndex = Math.min(
      this.items.length,
      Math.ceil((scrollTop + viewportHeight) / this.rowHeight) + this.overscan
    );

    this.content.style.transform = `translateY(${startIndex * this.rowHeight}px)`;

    const visibleItems = this.items.slice(startIndex, endIndex);
    let html = '';

    for (let i = 0; i < visibleItems.length; i++) {
      const item = visibleItems[i];
      const isSelected = this.selectionManager ? this.selectionManager.isSelected(item.relPath) : false;
      const statusMeta = window.FormatUtils.getStatusLabel(item.status);
      const isOnlyTarget = item.status === 'onlyInTarget';

      html += `
        <div class="table-row ${isSelected ? 'row-selected' : ''}" data-rel-path="${escapeHtml(item.relPath)}">
          <div class="td td-check">
            <input type="checkbox" class="row-checkbox" ${isSelected ? 'checked' : ''} ${isOnlyTarget ? 'disabled title="Arquivo não existe na origem"' : ''} />
          </div>
          <div class="td td-status">
            <span class="badge ${statusMeta.class}">${statusMeta.text}</span>
          </div>
          <div class="td td-path" title="${escapeHtml(item.relPath)}">
            ${escapeHtml(item.relPath)}
          </div>
          <div class="td td-size-src">
            ${window.FormatUtils.formatBytes(item.srcSize)}
          </div>
          <div class="td td-size-tgt">
            ${window.FormatUtils.formatBytes(item.targetSize)}
          </div>
          <div class="td td-date-src">
            ${window.FormatUtils.formatDate(item.srcMtime)}
          </div>
          <div class="td td-date-tgt">
            ${window.FormatUtils.formatDate(item.targetMtime)}
          </div>
        </div>
      `;
    }

    this.content.innerHTML = html;
    this.bindRowEvents();
  }

  bindRowEvents() {
    const rows = this.content.querySelectorAll('.table-row');
    rows.forEach((row) => {
      const relPath = row.getAttribute('data-rel-path');
      const checkbox = row.querySelector('.row-checkbox');

      checkbox.addEventListener('change', (e) => {
        e.stopPropagation();
        if (this.selectionManager) {
          this.selectionManager.toggle(relPath);
          this.onSelectionChange();
          this.render();
        }
      });

      row.addEventListener('click', (e) => {
        if (e.target.tagName.toLowerCase() === 'input') return;
        if (checkbox.disabled) return;
        if (this.selectionManager) {
          this.selectionManager.toggle(relPath);
          this.onSelectionChange();
          this.render();
        }
      });
    });
  }
}

function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

window.VirtualTable = VirtualTable;

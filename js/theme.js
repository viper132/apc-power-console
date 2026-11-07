/**
 * VIEW & COLOR MODE MANAGER
 * Supports:
 *   - Perspective View: 'gaming' (Reactor Core) vs 'formal' (Standard Enterprise UPS)
 *   - Color Mode: 'dark' vs 'light'
 *   - 12 APC Hardware Accent Colors (Gaming View)
 */

class ThemeManager {
  constructor() {
    this.currentView = localStorage.getItem('apc_view') || 'gaming';
    this.currentColorMode = localStorage.getItem('apc_colormode') || 'dark';
    this.currentColor = localStorage.getItem('apc_rgb_color') || 'cyan';

    this.colors = [
      { id: 'cyan', hex: '#00f0ff', label: 'Cyber Cyan' },
      { id: 'violet', hex: '#a855f7', label: 'Neon Violet' },
      { id: 'amber', hex: '#ffaa00', label: 'Reactor Amber' },
      { id: 'green', hex: '#10b981', label: 'Plasma Green' },
      { id: 'crimson', hex: '#ef4444', label: 'Crimson Hazard' },
      { id: 'arctic', hex: '#38bdf8', label: 'Arctic Blue' },
      { id: 'solar', hex: '#eab308', label: 'Solar Gold' },
      { id: 'pink', hex: '#ec4899', label: 'Laser Pink' },
      { id: 'cobalt', hex: '#3b82f6', label: 'Cobalt Deep' },
      { id: 'emerald', hex: '#059669', label: 'Emerald Matrix' },
      { id: 'orange', hex: '#f97316', label: 'Flare Orange' },
      { id: 'white', hex: '#e2e8f0', label: 'Ghost White' }
    ];

    this.init();
  }

  init() {
    this.applyView(this.currentView);
    this.applyColorMode(this.currentColorMode);
    this.applyColor(this.currentColor);
    this.renderColorPalette();
    this.bindEvents();
  }

  applyView(viewName) {
    this.currentView = viewName;
    document.body.setAttribute('data-view', viewName);
    localStorage.setItem('apc_view', viewName);

    // Update active pill in view switcher
    document.querySelectorAll('[data-view-btn]').forEach(btn => {
      btn.classList.toggle('active', btn.dataset.viewBtn === viewName);
    });

    const titleEl = document.getElementById('headerBrandTitle');
    if (titleEl) {
      titleEl.textContent = viewName === 'gaming' ? 'BACK-UPS PRO GAMING' : 'APC STANDARD POWER';
    }

    document.dispatchEvent(new CustomEvent('apc-view-changed', { detail: { view: viewName } }));
  }

  applyColorMode(mode) {
    this.currentColorMode = mode;
    document.body.setAttribute('data-color-mode', mode);
    localStorage.setItem('apc_colormode', mode);

    // Update active pill in color mode switcher
    document.querySelectorAll('[data-mode-btn]').forEach(btn => {
      btn.classList.toggle('active', btn.dataset.modeBtn === mode);
    });

    document.dispatchEvent(new CustomEvent('apc-mode-changed', { detail: { mode } }));
  }

  applyColor(colorId) {
    this.currentColor = colorId;
    document.body.setAttribute('data-accent-color', colorId);
    localStorage.setItem('apc_rgb_color', colorId);

    // Update active dot in palette
    document.querySelectorAll('.rgb-dot').forEach(dot => {
      dot.classList.toggle('active', dot.dataset.color === colorId);
    });

    document.dispatchEvent(new CustomEvent('apc-color-changed', { detail: { color: colorId } }));
  }

  renderColorPalette() {
    const bar = document.getElementById('rgbPaletteBar');
    if (!bar) return;

    bar.innerHTML = '';
    this.colors.forEach(c => {
      const dot = document.createElement('div');
      dot.className = `rgb-dot ${c.id === this.currentColor ? 'active' : ''}`;
      dot.dataset.color = c.id;
      dot.title = c.label;
      dot.style.backgroundColor = c.hex;
      dot.style.color = c.hex;

      dot.addEventListener('click', () => {
        this.applyColor(c.id);
        if (window.appBeeper) window.appBeeper.playClick(2800, 0.03);
      });

      bar.appendChild(dot);
    });
  }

  bindEvents() {
    // View Switcher Buttons
    document.querySelectorAll('[data-view-btn]').forEach(btn => {
      btn.addEventListener('click', () => {
        this.applyView(btn.dataset.viewBtn);
        if (window.appBeeper) window.appBeeper.playClick(1800, 0.04);
      });
    });

    // Dark / Light Mode Switcher Buttons
    document.querySelectorAll('[data-mode-btn]').forEach(btn => {
      btn.addEventListener('click', () => {
        this.applyColorMode(btn.dataset.modeBtn);
        if (window.appBeeper) window.appBeeper.playClick(2100, 0.04);
      });
    });
  }
}

window.ThemeManager = ThemeManager;

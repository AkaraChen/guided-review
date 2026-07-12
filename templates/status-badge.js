/* <status-badge> component: an uppercase mono pill for a genuine state
   (verification status, risk level, blocker). Set the `warn` attribute for
   the amber warning variant. */
(() => {
  if (customElements.get("status-badge")) return;

  class StatusBadgeElement extends HTMLElement {
    connectedCallback() {
      if (this.shadowRoot) return;
      const root = this.attachShadow({ mode: "open" });
      root.innerHTML = `
        <style>
          /* openknowledge chip: small rounded-full tag, hairline border first,
             quiet surface fill; warm amber tint reserved for warnings */
          :host {
            display: inline-flex;
            align-items: center;
            min-height: 22px;
            border: 1px solid var(--line);
            border-radius: 999px;
            background: var(--surface);
            color: var(--muted);
            padding: 1px 10px;
            font-family: var(--font-mono);
            font-size: 11px;
            font-weight: 500;
            letter-spacing: 0.02em;
            font-variant-numeric: tabular-nums;
          }

          :host([warn]) {
            border-color: var(--warn-line);
            background: var(--warn-fill);
            color: var(--warn-ink);
          }
        </style>
        <slot></slot>
      `;
    }
  }

  customElements.define("status-badge", StatusBadgeElement);
})();

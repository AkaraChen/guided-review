/* <claim-row> component: a status/level pill followed by slotted evidence and
   claim text. Set the `warn` attribute for the amber warning variant. */
(() => {
  if (customElements.get("claim-row")) return;

  class ClaimRowElement extends HTMLElement {
    connectedCallback() {
      if (this.shadowRoot) return;
      const root = this.attachShadow({ mode: "open" });
      root.innerHTML = `
        <style>
          :host {
            display: grid;
            grid-template-columns: 1fr;
            gap: 12px;
            border-top: 1px solid var(--line);
            padding-top: 16px;
          }

          slot {
            display: contents;
          }

          ::slotted(*) {
            min-width: 0;
          }

          ::slotted(p) {
            margin: 12px 0 0;
            color: var(--muted);
          }

          .tag {
            display: inline-flex;
            align-items: center;
            min-height: 26px;
            border-radius: 999px;
            background: var(--muted-fill);
            color: var(--ink);
            padding: 2px 12px;
            font-family: var(--font-mono);
            font-size: 11px;
            font-weight: 500;
            letter-spacing: 0.06em;
            text-transform: uppercase;
          }

          :host([warn]) .tag {
            background: var(--warn-fill);
            color: var(--warn-ink);
          }
        </style>
        <strong><span class="tag"></span></strong>
        <slot></slot>
      `;
      root.querySelector(".tag").textContent = this.getAttribute("label") ?? "";
    }
  }

  customElements.define("claim-row", ClaimRowElement);
})();

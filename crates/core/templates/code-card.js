/* <code-card> component: a code excerpt as a dark product window with a name
   and file-link header. The <pre> stays in the light DOM so the shiki
   enhancement (and its nested line spans) remain reachable from the page. */
(() => {
  if (customElements.get("code-card")) return;

  const lightDomStyle = document.createElement("style");
  lightDomStyle.textContent = `
    /* Document-level on purpose: for slotted elements the page's own rules
       (like the global h3 sizing) beat ::slotted rules from the shadow tree. */
    code-card h3 {
      /* Grows beside the file link; wraps the link to its own row when the
         card is too narrow for both. */
      flex: 1 1 16ch;
      min-width: 0;
      margin: 0;
      color: var(--code-ink);
      font-size: 13.5px;
      font-weight: 500;
      letter-spacing: 0;
    }

    code-card pre {
      margin: 0;
      overflow-x: auto;
      padding: 10px 0;
      font: 12.5px/1.65 var(--font-mono);
      /* Excerpts keep their tabs; 8-column tabs push code off narrow cards */
      tab-size: 4;
    }

    code-card .code-line {
      display: grid;
      grid-template-columns: 46px minmax(max-content, 1fr);
      gap: 14px;
      padding: 0 14px;
    }

    code-card .code-line:hover {
      background: rgba(240, 240, 238, 0.05);
    }

    code-card .ln {
      color: rgba(240, 240, 238, 0.48);
      font-variant-numeric: tabular-nums;
      text-align: right;
      user-select: none;
    }

    code-card .code-line:hover .ln {
      color: rgba(240, 240, 238, 0.8);
    }

    code-card .src {
      white-space: pre;
    }

    /* shiki enhancement: token colors come from inline styles; keep our window chrome */
    code-card pre.is-highlighted .src {
      color: #dbd7cf;
    }
  `;
  document.head.append(lightDomStyle);

  class CodeCardElement extends HTMLElement {
    connectedCallback() {
      if (this.shadowRoot) return;
      const root = this.attachShadow({ mode: "open" });
      root.innerHTML = `
        <style>
          :host {
            display: block;
            border: 1px solid var(--line-strong);
            border-radius: 12px;
            background: var(--code-bg);
            color: var(--code-ink);
            margin-top: 14px;
            overflow: hidden;
            box-shadow: var(--shadow-window);
          }

          header {
            display: flex;
            flex-wrap: wrap;
            align-items: center;
            justify-content: space-between;
            gap: 6px 16px;
            border-bottom: 1px solid rgba(240, 240, 238, 0.12);
            background: rgba(240, 240, 238, 0.03);
            padding: 11px 14px;
          }

          /* Slotted title and link participate in the header flexbox directly,
             so align-items centers them rather than the slot wrappers. */
          header slot {
            display: contents;
          }

          ::slotted(span[slot="link"]) {
            display: inline-flex;
            align-items: center;
            max-width: 100%;
            --file-link-color: #8ec6ff;
            --file-link-size: 12px;
          }
        </style>
        <header><slot name="name"></slot><slot name="link"></slot></header>
        <slot></slot>
      `;
    }
  }

  customElements.define("code-card", CodeCardElement);
})();
